use std::sync::mpsc::{Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

use crate::gimbal_controller::GimbalController;
use crate::movement::{self, MovementCommand};

pub fn run(receiver: Receiver<MovementCommand>) {
    let mut controller = GimbalController::new();

    let mut has_target = false;
    let mut last_update = Instant::now();
    let mut last_print = Instant::now();

    println!("Motor thread started!");

    loop {
        // Read pending commands and keep the latest one
        let mut latest_command = None;

        loop {
            match receiver.try_recv() {
                Ok(command) => {
                    latest_command = Some(command);
                }

                Err(TryRecvError::Empty) => {
                    break;
                }

                Err(TryRecvError::Disconnected) => {
                    controller.clear_target();
                    println!("Motor thread exiting.");
                    return;
                }
            }
        }

        // Apply the latest command
        if let Some(command) = latest_command {
            match command {
                MovementCommand::SetTarget(orientation) => {
                    movement::handle_orientation(
                        &mut controller,
                        orientation,
                    );

                    has_target = true;
                }

                MovementCommand::Stop => {
                    controller.clear_target();
                    has_target = false;
                }
            }
        }

        // Update all three motors
        let now = Instant::now();

        let dt_seconds =
            now.duration_since(last_update).as_secs_f32();

        controller.update(dt_seconds);

        last_update = now;

        // Print current motor positions every 200ms
        if last_print.elapsed() >= Duration::from_millis(200) {
            if has_target {
                controller.print_status();
            }

            last_print = Instant::now();
        }

        // Approx. 100 updates per second
        thread::sleep(Duration::from_millis(10));
    }
}
