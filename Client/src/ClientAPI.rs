
mod RotationMath;

#[cfg(feature = "vr")]
mod vr;

#[cfg(feature = "vr")]
mod mock_gimbal;

use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::thread;
use std::time::{Duration, Instant};
use self::RotationMath::{matrix_to_yaw_pitch_roll, degrees_to_steps};

const STEP_DEADZONE: i32 = 2;
const KEEPALIVE_INTERVAL: Duration = Duration::from_millis(100);

fn encode_packet(
    millis: u64,
    yaw: i32,
    pitch: i32,
    roll: i32,
) -> [u8; 20] {
    let mut packet = [0u8; 20];

    packet[0..8].copy_from_slice(&millis.to_le_bytes());
    packet[8..12].copy_from_slice(&yaw.to_le_bytes());
    packet[12..16].copy_from_slice(&pitch.to_le_bytes());
    packet[16..20].copy_from_slice(&roll.to_le_bytes());

    packet
}

fn filter_steps(
    current_steps: [i32; 3],
    last_sent_steps: Option<[i32; 3]>,
) -> [i32; 3] {
    let mut filtered_steps = current_steps;

    if let Some(last) = last_sent_steps {
        for i in 0..3 {
            let difference =
                (i64::from(current_steps[i]) - i64::from(last[i])).abs();

            if difference <= i64::from(STEP_DEADZONE) {
                filtered_steps[i] = last[i];
            }
        }
    }

    filtered_steps
}

fn should_send(
    filtered_steps: [i32; 3],
    last_sent_steps: Option<[i32; 3]>,
    last_sent_at: Option<Instant>,
    now: Instant,
) -> bool {
    let changed = last_sent_steps != Some(filtered_steps);

    let keepalive = last_sent_at.map_or(true, |time| {
        now.saturating_duration_since(time) >= KEEPALIVE_INTERVAL
    });

    changed || keepalive
}

fn send_orientation(
    socket: &UdpSocket,
    destination: SocketAddr,
    millis: u64,
    yaw: i32,
    pitch: i32,
    roll: i32,
) -> io::Result<usize> {
    let packet = encode_packet(millis, yaw, pitch, roll);

    socket.send_to(&packet, destination)
}

#[cfg(feature = "vr")]
fn main() {
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();

    let destination: SocketAddr =
        "10.219.103.53:5000".parse().unwrap();

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

        let filtered_steps = filter_steps(
            current_steps,
            last_sent_steps,
        );

        if should_send(
            filtered_steps,
            last_sent_steps,
            last_sent_at,
            Instant::now(),
        ) {
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

#[cfg(test)]
#[path = "../../tests/client/client_api_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "../../tests/client/rotation_math_tests.rs"]
mod rotation_tests;
