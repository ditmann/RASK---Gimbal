
use std::f64::consts::TAU;
use std::net::UdpSocket;
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

const SERVER_ADDRESS: &str = "10.219.103.53:5000";

const YAW_AMPLITUDE: f64 = 100.0;
const PITCH_AMPLITUDE: f64 = 20.0;
const ROLL_AMPLITUDE: f64 = 20.0;

const YAW_PERIOD: f64 = 2.0;
const PITCH_PERIOD: f64 = 2.0;
const ROLL_PERIOD: f64 = 2.0;

const UPDATE_INTERVAL_MS: u64 = 20;

fn oscillate(
    elapsed_seconds: f64,
    amplitude: f64,
    period: f64,
) -> i32 {
    let angle = TAU * elapsed_seconds / period;

    (angle.sin() * amplitude).round() as i32
}

fn create_packet(
    timestamp: u64,
    yaw: i32,
    pitch: i32,
    roll: i32,
) -> [u8; 20] {
    let mut packet = [0u8; 20];

    packet[0..8].copy_from_slice(&timestamp.to_le_bytes());
    packet[8..12].copy_from_slice(&yaw.to_le_bytes());
    packet[12..16].copy_from_slice(&pitch.to_le_bytes());
    packet[16..20].copy_from_slice(&roll.to_le_bytes());

    packet
}

fn main() -> std::io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect(SERVER_ADDRESS)?;

    let start = Instant::now();
    let mut last_log = Instant::now();

    println!("Mock Gimbal Client startet!");
    println!("Sender til: {}", SERVER_ADDRESS);
    println!("Trykk Ctrl+C for å stoppe.");

    loop {
        let elapsed = start.elapsed().as_secs_f64();

        let yaw = oscillate(
            elapsed,
            YAW_AMPLITUDE,
            YAW_PERIOD,
        );

        let pitch = oscillate(
            elapsed,
            PITCH_AMPLITUDE,
            PITCH_PERIOD,
        );

        let roll = oscillate(
            elapsed,
            ROLL_AMPLITUDE,
            ROLL_PERIOD,
        );

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        let packet = create_packet(
            timestamp,
            yaw,
            pitch,
            roll,
        );

        socket.send(&packet)?;

        if last_log.elapsed() >= Duration::from_millis(500) {
            println!(
                "Yaw: {:>4} | Pitch: {:>4} | Roll: {:>4}",
                yaw,
                pitch,
                roll
            );

            last_log = Instant::now();
        }

        thread::sleep(
            Duration::from_millis(UPDATE_INTERVAL_MS)
        );
    }
}
