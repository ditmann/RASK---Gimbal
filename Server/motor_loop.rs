
use std::sync::mpsc::{Receiver, RecvTimeoutError, TryRecvError};
use std::time::{Duration, Instant};

use crate::gimbal_controller::GimbalController;
use crate::movement::{self, MovementCommand};
use crate::orientation::Orientation;

const YAW_MAX_PULSES_PER_SECOND: f64 = 1000.0;
const PITCH_MAX_PULSES_PER_SECOND: f64 = 400.0;
const ROLL_MAX_PULSES_PER_SECOND: f64 = 400.0;

const YAW_START_PULSES_PER_SECOND: f64 = 1000.0;
const PITCH_START_PULSES_PER_SECOND: f64 = 50.0;
const ROLL_START_PULSES_PER_SECOND: f64 = 50.0;

const YAW_ACCELERATION: f64 = 1500.0;
const PITCH_ACCELERATION: f64 = 1500.0;
const ROLL_ACCELERATION: f64 = 1500.0;

const COMMAND_TIMEOUT: Duration = Duration::from_millis(200);

struct MotorRamp {
    speed: f64,
    direction: Option<bool>,
    next_step_at: Instant,
}

impl MotorRamp {
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

    fn step<F>(
        &mut self,
        remaining: i64,
        start_speed: f64,
        max_speed: f64,
        acceleration: f64,
        pulse: F,
    )
    where
        F: FnOnce(bool) -> i64,
    {
        if remaining == 0 {
            self.reset();
            return;
        }

        let wanted_forward = remaining > 0;

        if self.direction.is_none() {
            self.direction = Some(wanted_forward);
            self.speed = start_speed;
        }

        if self.direction != Some(wanted_forward)
            && self.speed <= start_speed
        {
            self.direction = Some(wanted_forward);
        }

        let forward = self.direction.unwrap();

        let remaining_after = pulse(forward);

        if remaining_after == 0 {
            self.reset();
            return;
        }

        let stopping_distance =
            (self.speed * self.speed - start_speed * start_speed)
                .max(0.0)
                / (2.0 * acceleration);

        let braking =
            (remaining_after > 0) != forward
                || (remaining_after.abs() as f64)
                <= stopping_distance + 1.0;

        if braking {
            self.speed =
                (self.speed * self.speed - 2.0 * acceleration)
                    .max(start_speed * start_speed)
                    .sqrt();
        } else {
            self.speed =
                (self.speed * self.speed + 2.0 * acceleration)
                    .sqrt()
                    .min(max_speed);
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

    let mut yaw = MotorRamp::new();
    let mut pitch = MotorRamp::new();
    let mut roll = MotorRamp::new();

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
            pitch.reset();
            roll.reset();

            last_target_at = None;
        } else if let Some(orientation) = latest_target {
            let yaw_was_moving = controller.yaw_is_moving();
            let pitch_was_moving = controller.pitch_is_moving();
            let roll_was_moving = controller.roll_is_moving();

            movement::handle_orientation(
                &mut controller,
                orientation,
            );

            last_target_at = Some(Instant::now());

            if !yaw_was_moving || !controller.yaw_is_moving() {
                yaw.reset();
            }

            if !pitch_was_moving || !controller.pitch_is_moving() {
                pitch.reset();
            }

            if !roll_was_moving || !controller.roll_is_moving() {
                roll.reset();
            }
        }

        if let Some(last) = last_target_at {
            if last.elapsed() >= COMMAND_TIMEOUT {
                controller.clear_target();

                yaw.reset();
                pitch.reset();
                roll.reset();

                last_target_at = None;

                println!("Motor stopped: command timeout.");
            }
        }

        if last_target_at.is_some() {
            let mut stepped = false;

            if controller.yaw_is_moving()
                && Instant::now() >= yaw.next_step_at
            {
                let remaining = controller.yaw_remaining_steps();

                yaw.step(
                    remaining,
                    YAW_START_PULSES_PER_SECOND,
                    YAW_MAX_PULSES_PER_SECOND,
                    YAW_ACCELERATION,
                    |forward| {
                        controller.pulse_yaw(forward);
                        controller.yaw_remaining_steps()
                    },
                );

                stepped = true;
            }

            if controller.pitch_is_moving()
                && Instant::now() >= pitch.next_step_at
            {
                let remaining = controller.pitch_remaining_steps();

                pitch.step(
                    remaining,
                    PITCH_START_PULSES_PER_SECOND,
                    PITCH_MAX_PULSES_PER_SECOND,
                    PITCH_ACCELERATION,
                    |forward| {
                        controller.pulse_pitch(forward);
                        controller.pitch_remaining_steps()
                    },
                );

                stepped = true;
            }

            if controller.roll_is_moving()
                && Instant::now() >= roll.next_step_at
            {
                let remaining = controller.roll_remaining_steps();
                roll.step(
                    remaining,
                    ROLL_START_PULSES_PER_SECOND,
                    ROLL_MAX_PULSES_PER_SECOND,
                    ROLL_ACCELERATION,
                    |forward| {
                        controller.pulse_roll(forward);
                        controller.roll_remaining_steps()
                    },
                );

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
                deadline = deadline.min(pitch.next_step_at);
            }

            if controller.roll_is_moving() {
                deadline = deadline.min(roll.next_step_at);
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
