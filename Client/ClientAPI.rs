mod RotationMath;
mod vr;

use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::time::{Duration, Instant};
use crate::RotationMath::{matrix_to_yaw_pitch_roll, degrees_to_steps};

const STEP_DEADZONE: i32 = 5;
const KEEPALIVE_INTERVAL: Duration = Duration::from_millis(100);

fn send_orientation(
    socket: &UdpSocket,
    destination: SocketAddr,
    millis: u64,
    yaw: i32,
    pitch: i32,
    roll: i32,
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
        "172.31.212.53:5000".parse().unwrap();

    socket.set_nonblocking(true).unwrap();

    let start = Instant::now();

    let mut last_sent_steps: Option<[i32; 3]> = None;
    let mut last_sent_at: Option<Instant> = None;

    let vr = vr::Vr::wait_for_steamvr();

    loop {
        // Get elapsed time
        let elapsed = start.elapsed();

        let _millis = elapsed.as_millis() as u64;
        let _time = elapsed.as_secs_f32();

        let Some(matrix) = vr.headset_matrix() else {
            thread::sleep(Duration::from_millis(20));
            continue;
        };

        //yaw will be 0 pitch 1 roll 2
        let _yaw_pitch_roll_steps: Vec<i32> =
            degrees_to_steps(matrix_to_yaw_pitch_roll(matrix));

        let _yaw: i32 = _yaw_pitch_roll_steps[0];
        let _pitch: i32 = _yaw_pitch_roll_steps[1];
        let _roll: i32 = _yaw_pitch_roll_steps[2];

        let current_steps = [_yaw, _pitch, _roll];
        let mut filtered_steps = current_steps;

        if let Some(last) = last_sent_steps {
            for i in 0..3 {
                if (current_steps[i] - last[i]).abs() <= STEP_DEADZONE {
                    filtered_steps[i] = last[i];
                }
            }
        }

        let changed = last_sent_steps != Some(filtered_steps);

        let keepalive = last_sent_at.map_or(true, |time| {
            time.elapsed() >= KEEPALIVE_INTERVAL
        });

        if changed || keepalive {
            match send_orientation(
                &socket,
                destination,
                _millis,
                filtered_steps[0],
                filtered_steps[1],
                filtered_steps[2],
            ) {
                Ok(_) => {
                    last_sent_steps = Some(filtered_steps);
                    last_sent_at = Some(Instant::now());

                    println!(
                        "Sent {_millis}ms | Y:{} P:{} R:{}",
                        filtered_steps[0],
                        filtered_steps[1],
                        filtered_steps[2]
                    );
                }

                Err(error) => {
                    println!("Send error: {error}");
                }
            }
        }

        // Approximately 50 updates per second
        thread::sleep(Duration::from_millis(20));
    }
}
