
use super::*;

fn orientation(yaw: i32, pitch: i32, roll: i32) -> Orientation {
    Orientation { yaw, pitch, roll }
}

#[test]
fn test_targets_are_applied_to_all_three_axes() {
    let mut controller = GimbalController::new();

    controller.set_target(orientation(100, -50, 25));

    assert_eq!(controller.yaw_remaining_steps(), 100);
    assert_eq!(controller.pitch_remaining_steps(), -50);
    assert_eq!(controller.roll_remaining_steps(), 25);

    assert!(controller.yaw_is_moving());
    assert!(controller.pitch_is_moving());
    assert!(controller.roll_is_moving());
}

#[test]
fn test_pulsing_one_axis_does_not_move_the_others() {
    let mut controller = GimbalController::new();

    controller.set_target(orientation(100, 50, -25));

    controller.pulse_yaw(true);
    controller.pulse_yaw(true);

    assert_eq!(controller.yaw_remaining_steps(), 98);
    assert_eq!(controller.pitch_remaining_steps(), 50);
    assert_eq!(controller.roll_remaining_steps(), -25);
}

#[test]
fn test_new_target_is_relative_to_current_position() {
    let mut controller = GimbalController::new();

    controller.set_target(orientation(100, 0, 0));

    for _ in 0..20 {
        controller.pulse_yaw(true);
    }

    controller.set_target(orientation(10, 0, 0));

    assert_eq!(controller.yaw_remaining_steps(), -10);

    for _ in 0..10 {
        controller.pulse_yaw(false);
    }

    assert_eq!(controller.yaw_remaining_steps(), 0);
    assert!(!controller.is_moving());
}

#[test]
fn test_all_axes_stop_requesting_steps_at_target() {
    let mut controller = GimbalController::new();

    controller.set_target(orientation(3, 2, -2));

    for _ in 0..3 {
        controller.pulse_yaw(true);
    }

    for _ in 0..2 {
        controller.pulse_pitch(true);
        controller.pulse_roll(false);
    }

    assert_eq!(controller.yaw_remaining_steps(), 0);
    assert_eq!(controller.pitch_remaining_steps(), 0);
    assert_eq!(controller.roll_remaining_steps(), 0);

    assert!(!controller.is_moving());
}

#[test]
fn test_clear_target_stops_without_losing_current_position() {
    let mut controller = GimbalController::new();

    controller.set_target(orientation(100, -20, 30));

    controller.pulse_yaw(true);
    controller.pulse_yaw(true);
    controller.pulse_pitch(false);
    controller.pulse_roll(true);

    controller.clear_target();

    assert!(!controller.is_moving());

    assert_eq!(controller.yaw_remaining_steps(), 0);
    assert_eq!(controller.pitch_remaining_steps(), 0);
    assert_eq!(controller.roll_remaining_steps(), 0);

    controller.set_target(orientation(5, -3, 2));

    assert_eq!(controller.yaw_remaining_steps(), 3);
    assert_eq!(controller.pitch_remaining_steps(), -2);
    assert_eq!(controller.roll_remaining_steps(), 1);
}
