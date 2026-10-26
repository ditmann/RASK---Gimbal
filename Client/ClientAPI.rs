
use std::net::UdpSocket;
use std::time::{Duration, Instant};
use std::thread;

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();

    let start = Instant::now();

    for i in 0..10 {
        let millis = start.elapsed().as_millis() as u64;

        let yaw: f32 = i as f32 * 5.0;
        let pitch: f32 = 20.0;
        let roll: f32 = 10.0;

        let mut packet = [0u8; 20];

        packet[0..8].copy_from_slice(&millis.to_le_bytes());
        packet[8..12].copy_from_slice(&yaw.to_le_bytes());
        packet[12..16].copy_from_slice(&pitch.to_le_bytes());
        packet[16..20].copy_from_slice(&roll.to_le_bytes());

        socket.send_to(&packet, "127.0.0.1:5000").unwrap();

        println!("Sent {millis}ms | Y:{yaw} P:{pitch} R:{roll}");
        thread::sleep(Duration::from_millis(1));

    }
}
