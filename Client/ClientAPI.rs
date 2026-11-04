mod validator;
mod vr;
mod matrixToDegrees;

use std::io::{self, ErrorKind};
use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::time::{Duration, Instant};
use crate::matrixToDegrees::Matrix_to_degrees;

fn send_orientation(
    socket: &UdpSocket,
    destination: SocketAddr,
    millis: u64,
    yaw: f32,
    pitch: f32,
    roll: f32,
) -> io::Result<usize> {
    let mut packet = [0u8; 20];

    packet[0..8].copy_from_slice(&millis.to_le_bytes());
    packet[8..12].copy_from_slice(&yaw.to_le_bytes());
    packet[12..16].copy_from_slice(&pitch.to_le_bytes());
    packet[16..20].copy_from_slice(&roll.to_le_bytes());

    socket.send_to(&packet, destination)
}

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();

    let destination: SocketAddr =
        "10.57.51.53:5000".parse().unwrap();

    socket.set_nonblocking(true).unwrap();

    let start = Instant::now();

    let mut last_status: Option<Instant> = None;
    let mut timeout_reported = false;

    let mut buffer = [0u8; 16];

    let vr = vr::Vr::wait_for_steamvr();


    loop {
        // Get elapsed time
        let elapsed = start.elapsed();

        let millis = elapsed.as_millis() as u64;
        let time = elapsed.as_secs_f32();

        let Some(matrix) = vr.headset_matrix() else {
            thread::sleep(Duration::from_millis(20));
            continue;
        };

        let yaw_pitch_roll: Vec<f32> = Matrix_to_degrees::matrix_to_yaw_pitch_roll(matrix);
        println!("THIS IS THE MATRIX:\n {matrix:?}");



        // Simulate movement on all three axes
        let yaw: f32 = 45.0 * (time * 5.0).sin();
        let pitch: f32 = 20.0;
        let roll: f32 = 10.0;

        // Validate before sending
        if validator::is_valid_orientation(yaw, pitch, roll) {
            match send_orientation(
                &socket,
                destination,
                millis,
                yaw,
                pitch,
                roll,
            ) {
                Ok(_) => {
                    println!(
                        "Sent {millis}ms | Y:{yaw:.2} P:{pitch:.2} R:{roll:.2}"
                    );
                }

                Err(error) => {
                    println!("Send error: {error}");
                }
            }
        } else {
            println!("Invalid orientation. Not sent.");
        }

        match socket.recv_from(&mut buffer) {
            Ok((size, address))
            if address == destination
                && &buffer[..size] == b"OK" =>
                {
                    if last_status.is_none() || timeout_reported {
                        println!("Server connected!");
                    }

                    last_status = Some(Instant::now());
                    timeout_reported = false;
                }

            Ok(_) => {
                // Ignore unknown responses
            }

            Err(error)
            if error.kind() == ErrorKind::WouldBlock
                || error.kind() == ErrorKind::ConnectionReset =>
                {
                }

            Err(error) => {
                println!("Status receive error: {error}");
            }
        }

        // Check server connection
        if last_status.unwrap_or(start).elapsed()
            >= Duration::from_secs(3)
            && !timeout_reported
        {
            println!("Server not responding!");
            timeout_reported = true;
        }

        // Approximately 50 updates per second
        thread::sleep(Duration::from_millis(20));
    }
}
