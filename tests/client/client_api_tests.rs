use super::*;

#[test]
fn test_packet_encoding() {
    let packet = encode_packet(
        0x0102030405060708,
        0x01020304,
        -2,
        i32::MIN,
    );

    assert_eq!(
        packet,
        [
            8, 7, 6, 5, 4, 3, 2, 1,
            4, 3, 2, 1,
            254, 255, 255, 255,
            0, 0, 0, 128,
        ]
    );
}

#[test]
fn test_deadzone() {
    let last = [100, -50, 20];

    assert_eq!(
        filter_steps([102, -52, 22], Some(last)),
        last
    );

    assert_eq!(
        filter_steps([103, -47, 17], Some(last)),
        [103, -47, 17]
    );

    assert_eq!(
        filter_steps([103, -51, 20], Some(last)),
        [103, -50, 20]
    );

    assert_eq!(
        filter_steps([100, -50, 20], None),
        [100, -50, 20]
    );
}

#[test]
fn test_keepalive() {
    let start = Instant::now();
    let steps = [100, 50, -20];

    assert!(should_send(
        steps,
        None,
        None,
        start,
    ));

    assert!(!should_send(
        steps,
        Some(steps),
        Some(start),
        start + Duration::from_millis(99),
    ));

    assert!(should_send(
        steps,
        Some(steps),
        Some(start),
        start + Duration::from_millis(100),
    ));
}

#[test]
fn test_deadzone_and_send_decision() {
    let start = Instant::now();
    let last = [100, 50, -20];

    let small_movement = filter_steps(
        [101, 52, -19],
        Some(last),
    );

    assert_eq!(small_movement, last);

    assert!(!should_send(
        small_movement,
        Some(last),
        Some(start),
        start + Duration::from_millis(20),
    ));

    let larger_movement = filter_steps(
        [103, 52, -19],
        Some(last),
    );

    assert_eq!(larger_movement, [103, 50, -20]);

    assert!(should_send(
        larger_movement,
        Some(last),
        Some(start),
        start + Duration::from_millis(20),
    ));
}
