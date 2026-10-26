
use std::net::UdpSocket;

fn main() {
    let socket = UdpSocket::bind("0.0.0.0:5000").unwrap();

    let mut buffer = [0u8; 20];
    let mut last_millis: Option<u64> = None;

    println!("Server running!");

    loop {
        let (size, address) = socket.recv_from(&mut buffer).unwrap();

        if size != 20 {
            println!("Invalid packet size: {size}");
            continue;
        }

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

        if let Some(previous) = last_millis {
            if millis <= previous {
                println!("Ignoring old packet: {millis}ms");
                continue;
            }
        }

        last_millis = Some(millis);

        println!(
            "From {address} | {millis}ms | Y:{yaw} P:{pitch} R:{roll}"
        );
    }
}
