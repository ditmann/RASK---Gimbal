
use std::io::{self, BufRead};
use std::net::UdpSocket;
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const SERVER_ADDRESS: &str = "10.219.103.53:5000";
const SEND_INTERVAL_MS: u64 = 50;


const YAW_LIMIT: i32 = 150;
const PITCH_LIMIT: i32 = 40;
const ROLL_LIMIT: i32 = 40;

#[derive(Clone, Copy, Default)]
struct Targets {
    yaw: i32,
    pitch: i32,
    roll: i32,
}

impl Targets {
    fn print(&self) {
        println!(
            "Target: Yaw {} | Pitch {} | Roll {}",
            self.yaw, self.pitch, self.roll
        );
    }
}

fn create_packet(targets: Targets) -> [u8; 20] {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64;

    let mut packet = [0u8; 20];

    packet[0..8].copy_from_slice(&timestamp.to_le_bytes());
    packet[8..12].copy_from_slice(&targets.yaw.to_le_bytes());
    packet[12..16].copy_from_slice(&targets.pitch.to_le_bytes());
    packet[16..20].copy_from_slice(&targets.roll.to_le_bytes());

    packet
}

fn move_axis(
    targets: &mut Targets,
    axis: &str,
    input: &str,
    absolute: bool,
) {
    let Ok(value) = input.parse::<i32>() else {
        println!("Invalid number: {}", input);
        return;
    };

    let (current, limit) = match axis {
        "yaw" => (&mut targets.yaw, YAW_LIMIT),
        "pitch" => (&mut targets.pitch, PITCH_LIMIT),
        "roll" => (&mut targets.roll, ROLL_LIMIT),
        _ => {
            println!("Unknown axis. Use yaw, pitch or roll.");
            return;
        }
    };

    let next = if absolute {
        Some(value)
    } else {
        current.checked_add(value)
    };

    let Some(next) = next else {
        println!("Number is too large.");
        return;
    };

    if next < -limit || next > limit {
        println!(
            "Target outside limit for {}: -{} to +{} steps",
            axis, limit, limit
        );
        return;
    }

    *current = next;
    targets.print();
}

fn print_help() {
    println!();
    println!("--- Manual Gimbal Controls ---");
    println!("yaw 20          Move yaw +20 steps");
    println!("yaw -20         Move yaw -20 steps");
    println!("pitch 10        Move pitch +10 steps");
    println!("roll -10        Move roll -10 steps");
    println!("set yaw 40      Set yaw target to 40");
    println!("set pitch 0     Set pitch target to 0");
    println!("zero            Set all targets to 0");
    println!("status          Show current targets");
    println!("help            Show commands");
    println!("quit            Exit");
    println!("------------------------------");
    println!();
}

fn main() -> io::Result<()> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect(SERVER_ADDRESS)?;

    // Read terminal input in another thread.
    // This lets the main thread keep sending UDP packets.
    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move || {
        let stdin = io::stdin();

        for line in stdin.lock().lines() {
            match line {
                Ok(line) => {
                    if tx.send(line).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    let mut targets = Targets::default();

    println!("Manual Gimbal Client started!");
    println!("Server: {}", SERVER_ADDRESS);
    println!("Sending targets every {} ms", SEND_INTERVAL_MS);

    print_help();
    targets.print();

    loop {
        match rx.recv_timeout(
            Duration::from_millis(SEND_INTERVAL_MS)
        ) {
            Ok(line) => {
                let parts: Vec<&str> =
                    line.split_whitespace().collect();

                match parts.as_slice() {
                    ["quit"] | ["exit"] => {
                        println!("Stopping manual client.");
                        break;
                    }

                    ["zero"] => {
                        targets = Targets::default();
                        targets.print();
                    }

                    ["status"] => {
                        targets.print();
                    }

                    ["help"] => {
                        print_help();
                    }

                    [axis, steps] => {
                        move_axis(
                            &mut targets,
                            axis,
                            steps,
                            false,
                        );
                    }

                    ["set", axis, steps] => {
                        move_axis(
                            &mut targets,
                            axis,
                            steps,
                            true,
                        );
                    }

                    [] => {}

                    _ => {
                        println!(
                            "Unknown command. Type 'help'."
                        );
                    }
                }
            }

            Err(mpsc::RecvTimeoutError::Timeout) => {}

            Err(mpsc::RecvTimeoutError::Disconnected) => {
                break;
            }
        }

        let packet = create_packet(targets);
        socket.send(&packet)?;
    }

    Ok(())
}
