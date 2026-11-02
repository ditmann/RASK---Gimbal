
use std::sync::mpsc::{Receiver, TryRecvError};
use std::thread;
use std::time::{Duration, Instant};

use crate::gimbal_controller::GimbalController;
use crate::movement::{self, MovementCommand};

pub fn run(receiver: Receiver<MovementCommand>) {
    // Create one persistent Gimbal Controller
    let mut controller = GimbalController::new();

    let mut has_target = false;
    let mut last_update = Instant::now();
    let mut last_print = Instant::now();

    println!("Motor thread started.");

    loop {
        let mut latest_target = None;
        let mut stop_requested = false;

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
                    println!("Motor thread disconnected.");
                    return;
                }
            }
        }


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

        let now = Instant::now();

        let dt_seconds =
            now.duration_since(last_update).as_secs_f32();

        controller.update(dt_seconds);

        last_update = now;

        if last_print.elapsed() >= Duration::from_millis(500) {
            if has_target {
                controller.print_status();
            }

            last_print = Instant::now();
        }

        thread::sleep(Duration::from_millis(1));
    }
}
