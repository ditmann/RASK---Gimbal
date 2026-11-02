mod server_api;
mod motor_loop;
mod movement;
mod orientation;
mod packet;
mod gimbal_controller;
mod motor_driver;

use std::sync::mpsc;
use std::thread;

use movement::MovementCommand;

fn main() {
    // Channel between networking and motor control
    let (sender, receiver) =
        mpsc::channel::<MovementCommand>();

    // Start the motor control thread
    let motor_thread = thread::spawn(move || {
        motor_loop::run(receiver);
    });

    // Run the UDP server on the main thread
    server_api::run(sender);

    // Wait for motor thread if the server exits
    motor_thread.join().expect("Motor thread panicked");
}
