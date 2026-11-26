
use super::*;

fn target(yaw: i32, pitch: i32, roll: i32) -> MovementCommand {
    MovementCommand::SetTarget(Orientation { yaw, pitch, roll })
}

#[test]
fn test_latest_target_replaces_older_targets() {
    let mut latest_target = None;
    let mut stop_requested = false;

    collect_command(
        target(100, 50, 25),
        &mut latest_target,
        &mut stop_requested,
    );

    collect_command(
        target(200, 0, -10),
        &mut latest_target,
        &mut stop_requested,
    );

    collect_command(
        target(-30, -20, 40),
        &mut latest_target,
        &mut stop_requested,
    );

    let last = latest_target.unwrap();

    assert_eq!(last.yaw, -30);
    assert_eq!(last.pitch, -20);
    assert_eq!(last.roll, 40);
    assert!(!stop_requested);
}

#[test]
fn test_stop_has_priority_regardless_of_command_order() {
    for stop_first in [true, false] {
        let mut latest_target = None;
        let mut stop_requested = false;

        if stop_first {
            collect_command(
                MovementCommand::Stop,
                &mut latest_target,
                &mut stop_requested,
            );

            collect_command(
                target(100, 20, 30),
                &mut latest_target,
                &mut stop_requested,
            );
        } else {
            collect_command(
                target(100, 20, 30),
                &mut latest_target,
                &mut stop_requested,
            );

            collect_command(
                MovementCommand::Stop,
                &mut latest_target,
                &mut stop_requested,
            );
        }

        assert!(stop_requested);
        assert!(latest_target.is_none());
    }
}

#[test]
fn test_watchdog_actually_stops_motors_and_resets_ramps() {
    let mut controller = GimbalController::new();

    controller.set_target(Orientation {
        yaw: 100,
        pitch: 50,
        roll: -20,
    });

    let mut yaw = MotorRamp::new();
    let mut pitch = MotorRamp::new();
    let mut roll = MotorRamp::new();

    let remaining = controller.yaw_remaining_steps();

    yaw.step(
        remaining,
        YAW_START_PULSES_PER_SECOND,
        YAW_MAX_PULSES_PER_SECOND,
        YAW_ACCELERATION,
        |forward| {
            controller.pulse_yaw(forward);
            controller.yaw_remaining_steps()
        },
    );

    assert!(yaw.speed > 0.0);

    let started = Instant::now();
    let mut last_target_at = Some(started);

    let stopped_early = stop_on_timeout(
        &mut controller,
        &mut yaw,
        &mut pitch,
        &mut roll,
        &mut last_target_at,
        started + Duration::from_millis(199),
    );

    assert!(!stopped_early);
    assert!(controller.is_moving());
    assert!(last_target_at.is_some());

    let stopped = stop_on_timeout(
        &mut controller,
        &mut yaw,
        &mut pitch,
        &mut roll,
        &mut last_target_at,
        started + Duration::from_millis(200),
    );

    assert!(stopped);
    assert!(!controller.is_moving());

    assert_eq!(yaw.speed, 0.0);
    assert_eq!(pitch.speed, 0.0);
    assert_eq!(roll.speed, 0.0);

    assert!(last_target_at.is_none());
}

#[test]
fn test_new_command_refreshes_watchdog() {
    let mut controller = GimbalController::new();

    controller.set_target(Orientation {
        yaw: 100,
        pitch: 0,
        roll: 0,
    });

    let mut yaw = MotorRamp::new();
    let mut pitch = MotorRamp::new();
    let mut roll = MotorRamp::new();

    let started = Instant::now();
    let refreshed = started + Duration::from_millis(150);

    let mut last_target_at = Some(refreshed);

    let stopped = stop_on_timeout(
        &mut controller,
        &mut yaw,
        &mut pitch,
        &mut roll,
        &mut last_target_at,
        started + Duration::from_millis(250),
    );

    assert!(!stopped);
    assert!(controller.is_moving());
    assert_eq!(last_target_at, Some(refreshed));
}
