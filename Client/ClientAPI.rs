
use std::net::UdpSocket;
use std::time::{Duration, Instant};
use std::thread;

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();

    let start = Instant::now();
    let mut yaw: f32 = 0.0;

    loop {
        let millis = start.elapsed().as_millis() as u64;

        let pitch: f32 = 20.0;
        let roll: f32 = 10.0;

        if yaw.is_finite() && pitch.is_finite() && roll.is_finite() {
            let mut packet = [0u8; 20];

            packet[0..8].copy_from_slice(&millis.to_le_bytes());
            packet[8..12].copy_from_slice(&yaw.to_le_bytes());
            packet[12..16].copy_from_slice(&pitch.to_le_bytes());
            packet[16..20].copy_from_slice(&roll.to_le_bytes());

            socket.send_to(&packet, "127.0.0.1:5000").unwrap();

            println!("Sent {millis}ms | Y:{yaw} P:{pitch} R:{roll}");
        } else {
            println!("Invalid orientation! Packet not sent.");
        }

        // Simulated VR rotation
        yaw += 1.0;

        if yaw > 45.0 {
            yaw = -45.0;
        }

        thread::sleep(Duration::from_millis(20));
    }
}
