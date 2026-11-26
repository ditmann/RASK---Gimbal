use std::io::ErrorKind;
use std::net::UdpSocket;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};

use crate::movement::MovementCommand;
use crate::packet::OrientationPacket;

pub fn run(sender: Sender<MovementCommand>) {
    let socket = UdpSocket::bind("0.0.0.0:5000").unwrap();

    socket.set_read_timeout(
        Some(Duration::from_millis(50))
    ).unwrap();

    let mut buffer = [0u8; 64];

    let mut last_millis: Option<u64> = None;
    let mut last_packet_time: Option<Instant> = None;
    let mut last_status_sent: Option<Instant> = None;

    let mut connection_lost = false;

    println!("Server listening on UDP port 5000...");

    loop {
        // Receive and validate UDP
        match socket.recv_from(&mut buffer) {
            Ok((size, address)) => {
                if let Some(packet) =
                    OrientationPacket::decode(&buffer[..size])
                {
                    // Reject older packets
                    let is_new = match last_millis {
                        Some(previous) => packet.millis > previous,
                        None => true,
                    };

                    if is_new {
                        // Send new target to motor thread
                        if sender.send(
                            MovementCommand::SetTarget(packet.orientation)
                        ).is_err() {
                            println!("Motor thread disconnected!");
                            return;
                        }

                        last_millis = Some(packet.millis);
                        last_packet_time = Some(Instant::now());

                        if connection_lost {
                            println!("Connection restored!");
                            connection_lost = false;
                        }

                        // Send "I am alive" about once per second
                        let send_status = match last_status_sent {
                            Some(previous) => {
                                previous.elapsed() >= Duration::from_secs(1)
                            }
                            None => true,
                        };

                        if send_status {
                            match socket.send_to(b"OK", address) {
                                Ok(_) => {
                                    last_status_sent = Some(Instant::now());
                                }
                                Err(error) => {
                                    println!("Heartbeat error: {error}");
                                }
                            }
                        }
                    } else {
                        println!(
                            "Old packet ignored: {}ms",
                            packet.millis
                        );
                    }
                } else {
                    println!("Invalid UDP packet ({size} bytes)");
                }
            }

            Err(error) if error.kind() == ErrorKind::TimedOut
                || error.kind() == ErrorKind::WouldBlock => {
                // No packet received
            }

            Err(error) => {
                println!("UDP error: {error}");
            }
        }

        // Check connection watchdog
        if let Some(last_time) = last_packet_time {
            if last_time.elapsed() >= Duration::from_millis(200)
                && !connection_lost
            {
                println!("Connection lost!");

                connection_lost = true;
                last_millis = None;
                last_status_sent = None;

                // Tell motor thread to hold position
                if sender.send(MovementCommand::Stop).is_err() {
                    println!("Motor thread disconnected!");
                    return;
                }
            }
        }
    }
}
