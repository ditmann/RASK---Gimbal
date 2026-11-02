
use std::sync::mpsc::{
    Receiver,
    RecvTimeoutError,
    TryRecvError,
};

use std::time::{Duration, Instant};

use crate::gimbal_controller::GimbalController;
use crate::movement::{self, MovementCommand};
use crate::orientation::Orientation;

// Maximum requested step rate
const MAX_PULSES_PER_SECOND: u64 = 800;

const STEP_INTERVAL: Duration =
    Duration::from_micros(
        1_000_000 / MAX_PULSES_PER_SECOND
    );

// Stop movement if fresh targets stop arriving
const COMMAND_TIMEOUT: Duration = Duration::from_millis(200);

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
    let mut last_update = Instant::now();
    let mut last_print = Instant::now();

    let mut next_step_at = Instant::now();

    // Stores a command received while waiting
    let mut pending_command: Option<MovementCommand> = None;

    println!("Motor thread started.");

    loop {
        let mut latest_target = None;
        let mut stop_requested = false;

        // Process the command that woke us up
        if let Some(command) = pending_command.take() {
            collect_command(
                command,
                &mut latest_target,
                &mut stop_requested,
            );
        }

        // Drain any additional waiting commands
        loop {
            match receiver.try_recv() {
                Ok(command) => {
                    collect_command(
                        command,
                        &mut latest_target,
                        &mut stop_requested,
                    );
                }

                Err(TryRecvError::Empty) => {
                    break;
                }

                Err(TryRecvError::Disconnected) => {
                    controller.clear_target();
                    return;
                }
            }
        }

        // Apply Stop or the latest target
        if stop_requested {
            controller.clear_target();
            last_target_at = None;
        } else if let Some(orientation) = latest_target {
            let was_moving = controller.is_moving();

            movement::handle_orientation(
                &mut controller,
                orientation,
            );

            last_target_at = Some(Instant::now());

            // Start immediately if previously idle
            if !was_moving {
                next_step_at = Instant::now();
            }
        }

        // Check communication watchdog
        if let Some(last) = last_target_at {
            if last.elapsed() >= COMMAND_TIMEOUT {
                controller.clear_target();
                last_target_at = None;

                println!("Motor stopped: command timeout.");
            }
        }

        // Generate a STEP pulse when due
        if last_target_at.is_some() && controller.is_moving() {
            let now = Instant::now();

            if now >= next_step_at {
                let dt_seconds =
                    now.duration_since(last_update).as_secs_f32();

                last_update = now;

                controller.update(dt_seconds);

                // Schedule the next pulse from the
                // start of this pulse, not from the
                // previous deadline.
                next_step_at = now + STEP_INTERVAL;

                if last_print.elapsed()
                    >= Duration::from_millis(500)
                {
                    controller.print_status();
                    last_print = Instant::now();
                }

                continue;
            }
        }

        // Calculate how long we can wait
        if let Some(last) = last_target_at {
            let watchdog_deadline =
                last + COMMAND_TIMEOUT;

            let deadline = if controller.is_moving() {
                next_step_at.min(watchdog_deadline)
            } else {
                watchdog_deadline
            };

            let wait_time =
                deadline.saturating_duration_since(
                    Instant::now()
                );

            // Wake when a command arrives OR a
            // motor/watchdog deadline is reached.
            match receiver.recv_timeout(wait_time) {
                Ok(command) => {
                    pending_command = Some(command);
                }

                Err(RecvTimeoutError::Timeout) => {
                    // Check deadlines again
                }

                Err(RecvTimeoutError::Disconnected) => {
                    controller.clear_target();
                    return;
                }
            }
        } else {
            // Fully idle: block until a command arrives
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
