
use std::sync::mpsc::{Receiver, RecvTimeoutError, TryRecvError};
use std::time::{Duration, Instant};

use crate::gimbal_controller::GimbalController;
use crate::movement::{self, MovementCommand};
use crate::orientation::Orientation;

const YAW_MAX_PULSES_PER_SECOND: f64 = 1200.0;
const YAW_START_PULSES_PER_SECOND: f64 = 100.0;
const YAW_ACCELERATION: f64 = 5000.0;

const PITCH_PULSES_PER_SECOND: u64 = 400;
const ROLL_PULSES_PER_SECOND: u64 = 400;

const PITCH_STEP_INTERVAL: Duration =
    Duration::from_micros(1_000_000 / PITCH_PULSES_PER_SECOND);

const ROLL_STEP_INTERVAL: Duration =
    Duration::from_micros(1_000_000 / ROLL_PULSES_PER_SECOND);

const COMMAND_TIMEOUT: Duration = Duration::from_millis(200);

struct YawRamp {
    speed: f64,
    direction: Option<bool>,
    next_step_at: Instant,
}

impl YawRamp {
    fn new() -> Self {
        Self {
            speed: 0.0,
            direction: None,
            next_step_at: Instant::now(),
        }
    }

    fn reset(&mut self) {
        self.speed = 0.0;
        self.direction = None;
        self.next_step_at = Instant::now();
    }

    fn step(&mut self, controller: &mut GimbalController) {
        let remaining = controller.yaw_remaining_steps();

        if remaining == 0 {
            self.reset();
            return;
        }

        let wanted_forward = remaining > 0;

        if self.direction.is_none() {
            self.direction = Some(wanted_forward);
            self.speed = YAW_START_PULSES_PER_SECOND;
        }

        if self.direction != Some(wanted_forward)
            && self.speed <= YAW_START_PULSES_PER_SECOND
        {
            self.direction = Some(wanted_forward);
        }

        let forward = self.direction.unwrap();

        controller.pulse_yaw(forward);

        let remaining = controller.yaw_remaining_steps();

        if remaining == 0 {
            self.reset();
            return;
        }

        let stopping_distance = (
            self.speed * self.speed
                - YAW_START_PULSES_PER_SECOND
                * YAW_START_PULSES_PER_SECOND
        )
            .max(0.0)
            / (2.0 * YAW_ACCELERATION);

        let braking = (remaining > 0) != forward
            || (remaining.abs() as f64) <= stopping_distance + 1.0;

        if braking {
            self.speed = (
                self.speed * self.speed
                    - 2.0 * YAW_ACCELERATION
            )
                .max(
                    YAW_START_PULSES_PER_SECOND
                        * YAW_START_PULSES_PER_SECOND,
                )
                .sqrt();
        } else {
            self.speed = (
                self.speed * self.speed
                    + 2.0 * YAW_ACCELERATION
            )
                .sqrt()
                .min(YAW_MAX_PULSES_PER_SECOND);
        }

        self.next_step_at = Instant::now()
            + Duration::from_secs_f64(1.0 / self.speed);
    }
}

fn collect_command(
    command: MovementCommand,
    latest_target: &mut Option<Orientation>,
    stop_requested: &mut bool,
) {
    match command {
        MovementCommand::SetTarget(orientation) => {
            if !*stop_requested {
                *latest_target = Some(orientation);
            }
        }

        MovementCommand::Stop => {
            *stop_requested = true;
            *latest_target = None;
        }
    }
}

pub fn run(receiver: Receiver<MovementCommand>) {
    let mut controller = GimbalController::new();

    let mut last_target_at: Option<Instant> = None;
    let mut last_print = Instant::now();

    let mut yaw = YawRamp::new();

    let mut next_pitch_step_at = Instant::now();
    let mut next_roll_step_at = Instant::now();

    let mut last_pitch_update = Instant::now();
    let mut last_roll_update = Instant::now();

    let mut pending_command: Option<MovementCommand> = None;

    println!("Motor thread started.");

    loop {
        let mut latest_target = None;
        let mut stop_requested = false;

        if let Some(command) = pending_command.take() {
            collect_command(
                command,
                &mut latest_target,
                &mut stop_requested,
            );
        }

        loop {
            match receiver.try_recv() {
                Ok(command) => {
                    collect_command(
                        command,
                        &mut latest_target,
                        &mut stop_requested,
                    );
                }

                Err(TryRecvError::Empty) => break,

                Err(TryRecvError::Disconnected) => {
                    controller.clear_target();
                    return;
                }
            }
        }

        if stop_requested {
            controller.clear_target();
            yaw.reset();
            last_target_at = None;
        } else if let Some(orientation) = latest_target {
            let yaw_was_moving = controller.yaw_is_moving();
            let pitch_was_moving = controller.pitch_is_moving();
            let roll_was_moving = controller.roll_is_moving();

            movement::handle_orientation(
                &mut controller,
                orientation,
            );

            let now = Instant::now();
            last_target_at = Some(now);

            if !yaw_was_moving || !controller.yaw_is_moving() {
                yaw.reset();
            }

            if !pitch_was_moving && controller.pitch_is_moving() {
                next_pitch_step_at = now;
                last_pitch_update = now;
            }

            if !roll_was_moving && controller.roll_is_moving() {
                next_roll_step_at = now;
                last_roll_update = now;
            }
        }

        if let Some(last) = last_target_at {
            if last.elapsed() >= COMMAND_TIMEOUT {
                controller.clear_target();
                yaw.reset();
                last_target_at = None;

                println!("Motor stopped: command timeout.");
            }
        }

        if last_target_at.is_some() {
            let now = Instant::now();
            let mut stepped = false;

            if controller.yaw_is_moving()
                && now >= yaw.next_step_at
            {
                yaw.step(&mut controller);
                stepped = true;
            }

            if controller.pitch_is_moving()
                && now >= next_pitch_step_at
            {
                let dt = now
                    .duration_since(last_pitch_update)
                    .as_secs_f32();

                controller.update_pitch(dt);

                last_pitch_update = now;
                next_pitch_step_at =
                    Instant::now() + PITCH_STEP_INTERVAL;

                stepped = true;
            }

            if controller.roll_is_moving()
                && now >= next_roll_step_at
            {
                let dt = now
                    .duration_since(last_roll_update)
                    .as_secs_f32();

                controller.update_roll(dt);

                last_roll_update = now;
                next_roll_step_at =
                    Instant::now() + ROLL_STEP_INTERVAL;

                stepped = true;
            }

            if stepped {
                if last_print.elapsed() >= Duration::from_millis(500) {
                    controller.print_status();
                    last_print = Instant::now();
                }

                continue;
            }
        }

        if let Some(last) = last_target_at {
            let mut deadline = last + COMMAND_TIMEOUT;

            if controller.yaw_is_moving() {
                deadline = deadline.min(yaw.next_step_at);
            }

            if controller.pitch_is_moving() {
                deadline = deadline.min(next_pitch_step_at);
            }

            if controller.roll_is_moving() {
                deadline = deadline.min(next_roll_step_at);
            }

            let wait_time =
                deadline.saturating_duration_since(Instant::now());

            match receiver.recv_timeout(wait_time) {
                Ok(command) => {
                    pending_command = Some(command);
                }

                Err(RecvTimeoutError::Timeout) => {}

                Err(RecvTimeoutError::Disconnected) => {
                    controller.clear_target();
                    return;
                }
            }
        } else {
            match receiver.recv() {
                Ok(command) => {
                    pending_command = Some(command);
                }

                Err(_) => {
                    controller.clear_target();
                    return;
                }
            }
        }
    }
}
