
use super::*;

fn create_packet(
    millis: u64,
    yaw: i32,
    pitch: i32,
    roll: i32,
) -> [u8; 20] {
    let mut bytes = [0u8; 20];

    bytes[0..8].copy_from_slice(&millis.to_le_bytes());
    bytes[8..12].copy_from_slice(&yaw.to_le_bytes());
    bytes[12..16].copy_from_slice(&pitch.to_le_bytes());
    bytes[16..20].copy_from_slice(&roll.to_le_bytes());

    bytes
}

#[test]
fn test_valid_packet() {
    let bytes = create_packet(1000, 100, 200, 300);

    let packet = OrientationPacket::decode(&bytes)
        .expect("Valid packet should decode");

    assert_eq!(packet.millis, 1000);
    assert_eq!(packet.orientation.yaw, 100);
    assert_eq!(packet.orientation.pitch, 200);
    assert_eq!(packet.orientation.roll, 300);
}

#[test]
fn test_negative_positions() {
    let bytes = create_packet(2000, -100, -200, -300);

    let packet = OrientationPacket::decode(&bytes)
        .expect("Negative positions should decode");

    assert_eq!(packet.orientation.yaw, -100);
    assert_eq!(packet.orientation.pitch, -200);
    assert_eq!(packet.orientation.roll, -300);
}

#[test]
fn test_zero_positions() {
    let bytes = create_packet(0, 0, 0, 0);

    let packet = OrientationPacket::decode(&bytes)
        .expect("Zero values should decode");

    assert_eq!(packet.millis, 0);
    assert_eq!(packet.orientation.yaw, 0);
    assert_eq!(packet.orientation.pitch, 0);
    assert_eq!(packet.orientation.roll, 0);
}

#[test]
fn test_max_min_values() {
    let bytes = create_packet(
        u64::MAX,
        i32::MAX,
        i32::MIN,
        -1,
    );

    let packet = OrientationPacket::decode(&bytes)
        .expect("Extreme values should decode");

    assert_eq!(packet.millis, u64::MAX);
    assert_eq!(packet.orientation.yaw, i32::MAX);
    assert_eq!(packet.orientation.pitch, i32::MIN);
    assert_eq!(packet.orientation.roll, -1);
}

#[test]
fn test_packet_too_short() {
    for length in 0..20 {
        let bytes = vec![0u8; length];

        assert!(
            OrientationPacket::decode(&bytes).is_none(),
            "Packet with {} bytes should be rejected",
            length
        );
    }
}

#[test]
fn test_packet_too_long() {
    let bytes = vec![0u8; 21];

    assert!(OrientationPacket::decode(&bytes).is_none());
}

#[test]
fn test_little_endian_decoding() {
    let mut bytes = [0u8; 20];

    bytes[0..8].copy_from_slice(&[
        0x08, 0x07, 0x06, 0x05,
        0x04, 0x03, 0x02, 0x01,
    ]);

    bytes[8..12].copy_from_slice(&[1, 0, 0, 0]);
    bytes[12..16].copy_from_slice(&[255, 255, 255, 255]);
    bytes[16..20].copy_from_slice(&[0, 1, 0, 0]);

    let packet = OrientationPacket::decode(&bytes)
        .expect("Packet should decode");

    assert_eq!(packet.millis, 0x0102030405060708);
    assert_eq!(packet.orientation.yaw, 1);
    assert_eq!(packet.orientation.pitch, -1);
    assert_eq!(packet.orientation.roll, 256);
}
