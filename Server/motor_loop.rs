
use std::sync::mpsc::{
    Receiver,
    RecvTimeoutError,
    TryRecvError,
};

use std::time::{Duration, Instant};

use crate::gimbal_controller::GimbalController;
use crate::movement::{self, MovementCommand};

pub fn run(receiver: Receiver<MovementCommand>) {
    let mut controller = GimbalController::new();

    let mut has_target = false;
    let mut last_update = Instant::now();
    let mut last_print = Instant::now();

    println!("Motor thread started.");

    loop {
        let mut latest_target = None;
        let mut stop_requested = false;

        // 1. Wait for a new command, or wake after 1 ms
        match receiver.recv_timeout(Duration::from_millis(1)) {
            Ok(MovementCommand::SetTarget(orientation)) => {
                latest_target = Some(orientation);
            }

            Ok(MovementCommand::Stop) => {
                stop_requested = true;
            }

            Err(RecvTimeoutError::Timeout) => {
                // No command. Continue motor movement.
            }

            Err(RecvTimeoutError::Disconnected) => {
                controller.clear_target();
                return;
            }
        }

        // 2. Collect any additional waiting commands
        loop {
            match receiver.try_recv() {
                Ok(MovementCommand::SetTarget(orientation)) => {
                    if !stop_requested {
                        latest_target = Some(orientation);
                    }
                }

                Ok(MovementCommand::Stop) => {
                    stop_requested = true;
                    latest_target = None;
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

        // 3. Process the newest target or stop command
        if stop_requested {
            controller.clear_target();
            has_target = false;
        } else if let Some(orientation) = latest_target {
            movement::handle_orientation(
                &mut controller,
                orientation,
            );

            has_target = true;
        }

        // 4. Update the physical motor
        let now = Instant::now();

        let dt_seconds =
            now.duration_since(last_update).as_secs_f32();

        last_update = now;

        controller.update(dt_seconds);

        // 5. Print status occasionally
        if last_print.elapsed() >= Duration::from_millis(500) {
            if has_target {
                controller.print_status();
            }

            last_print = Instant::now();
        }
    }
}
