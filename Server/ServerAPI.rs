
use std::net::UdpSocket;
use std::time::{Duration, Instant};
use std::io::ErrorKind;

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:5000").unwrap();

    socket.set_read_timeout(
        Some(Duration::from_millis(50))
    ).unwrap();

    let mut buffer = [0u8; 64];

    let mut last_millis: Option<u64> = None;
    let mut last_packet_time: Option<Instant> = None;
    let mut connection_lost = false;

    println!("Server listening...");

    loop {
        match socket.recv_from(&mut buffer) {
            Ok((size, address)) => {

                if size != 20 {
                    println!("Invalid packet size: {size}");
                } else {
                    // Decode packet
                    let millis = u64::from_le_bytes(
                        buffer[0..8].try_into().unwrap()
                    );

                    let yaw = f32::from_le_bytes(
                        buffer[8..12].try_into().unwrap()
                    );

                    let pitch = f32::from_le_bytes(
                        buffer[12..16].try_into().unwrap()
                    );

                    let roll = f32::from_le_bytes(
                        buffer[16..20].try_into().unwrap()
                    );

                    let valid = yaw.is_finite()
                        && pitch.is_finite()
                        && roll.is_finite();

                    if !valid {
                        println!("Invalid orientation received!");
                    } else {
                        let is_new = match last_millis {
                            Some(previous) => millis > previous,
                            None => true,
                        };

                        if is_new {
                            last_millis = Some(millis);
                            last_packet_time = Some(Instant::now());

                            if connection_lost {
                                println!("Connection restored!");
                                connection_lost = false;
                            }

                            println!(
                                "From {address} | {millis}ms | Y:{yaw} P:{pitch} R:{roll}"
                            );
                        } else {
                            println!("Old packet ignored: {millis}ms");
                        }
                    }
                }
            }

            Err(error) if error.kind() == ErrorKind::TimedOut
                || error.kind() == ErrorKind::WouldBlock => {
            }

            Err(error) => {
                println!("UDP error: {error}");
            }
        }

        // need session id?
        if let Some(last_time) = last_packet_time {
            if last_time.elapsed() >= Duration::from_millis(200)
                && !connection_lost
            {
                println!("Connection lost!");
                connection_lost = true;

                last_millis = None;
            }
        }
    }
}
