
use super::*;
use crate::motor_driver::{HardwarePitch, HardwareRoll, HardwareYaw};

fn step_yaw(ramp: &mut MotorRamp, motor: &mut HardwareYaw) {
    let remaining = motor.remaining_steps();
    ramp.step(
        remaining,
        YAW_START_PULSES_PER_SECOND,
        YAW_MAX_PULSES_PER_SECOND,
        YAW_ACCELERATION,
        |forward| {
            motor.step_in_direction(forward);
            motor.remaining_steps()
        },
    );
}

fn step_pitch(ramp: &mut MotorRamp, motor: &mut HardwarePitch) {
    let remaining = motor.remaining_steps();
    ramp.step(
        remaining,
        PITCH_START_PULSES_PER_SECOND,
        PITCH_MAX_PULSES_PER_SECOND,
        PITCH_ACCELERATION,
        |forward| {
            motor.step_in_direction(forward);
            motor.remaining_steps()
        },
    );
}

fn step_roll(ramp: &mut MotorRamp, motor: &mut HardwareRoll) {
    let remaining = motor.remaining_steps();
    ramp.step(
        remaining,
        ROLL_START_PULSES_PER_SECOND,
        ROLL_MAX_PULSES_PER_SECOND,
        ROLL_ACCELERATION,
        |forward| {
            motor.step_in_direction(forward);
            motor.remaining_steps()
        },
    );
}

#[test]
fn test_no_pulse_when_target_reached() {
    let mut ramp = MotorRamp::new();
    let mut pulses = 0;

    ramp.step(0, 100.0, 1000.0, 1500.0, |_| {
        pulses += 1;
        0
    });

    assert_eq!(pulses, 0);
    assert_eq!(ramp.speed, 0.0);
    assert_eq!(ramp.direction, None);
}

#[test]
fn test_short_movement_reaches_target() {
    let mut ramp = MotorRamp::new();
    let mut motor = HardwareYaw::new().unwrap();
    motor.set_target_step(3);

    let mut pulses = 0;

    while motor.is_moving() && pulses < 10 {
        step_yaw(&mut ramp, &mut motor);
        pulses += 1;
    }

    assert_eq!(pulses, 3);
    assert_eq!(motor.remaining_steps(), 0);
    assert!(!motor.is_moving());
    assert_eq!(ramp.speed, 0.0);
}

#[test]
fn test_long_movement_accelerates_brakes_and_stays_within_limit() {
    let mut ramp = MotorRamp::new();
    let mut motor = HardwareYaw::new().unwrap();
    motor.set_target_step(2000);

    let mut last_speed = 0.0;
    let mut reached_max = false;
    let mut accelerated = false;
    let mut decelerated = false;
    let mut pulses = 0;

    while motor.is_moving() && pulses < 2100 {
        step_yaw(&mut ramp, &mut motor);
        pulses += 1;

        if motor.is_moving() {
            assert!(ramp.speed >= YAW_START_PULSES_PER_SECOND);
            assert!(ramp.speed <= YAW_MAX_PULSES_PER_SECOND);

            accelerated |= ramp.speed > last_speed;
            decelerated |= ramp.speed < last_speed;
            reached_max |=
                (ramp.speed - YAW_MAX_PULSES_PER_SECOND).abs() < 1e-6;

            last_speed = ramp.speed;
        }
    }

    assert_eq!(pulses, 2000);
    assert!(!motor.is_moving());
    assert!(accelerated);
    assert!(reached_max);
    assert!(decelerated);
    assert_eq!(ramp.speed, 0.0);
}

#[test]
fn test_retarget_in_opposite_direction_brakes_before_reversing() {
    let mut ramp = MotorRamp::new();
    let mut motor = HardwareYaw::new().unwrap();
    motor.set_target_step(1000);

    for _ in 0..80 {
        step_yaw(&mut ramp, &mut motor);
    }

    assert!(ramp.speed > YAW_START_PULSES_PER_SECOND);

    motor.set_target_step(-20);

    let mut reversed = false;
    let mut pulses = 0;

    while motor.is_moving() && pulses < 1000 {
        let speed_before = ramp.speed;
        let direction_before = ramp.direction;

        step_yaw(&mut ramp, &mut motor);
        pulses += 1;

        if direction_before == Some(true)
            && ramp.direction == Some(false)
        {
            assert!(
                speed_before <=
                    YAW_START_PULSES_PER_SECOND + 1e-6
            );

            reversed = true;
        }
    }

    assert!(reversed);
    assert!(!motor.is_moving());
    assert_eq!(motor.remaining_steps(), 0);
}

#[test]
fn test_pitch_and_roll_obey_individual_limits() {
    let mut pitch_ramp = MotorRamp::new();
    let mut roll_ramp = MotorRamp::new();

    let mut pitch = HardwarePitch::new().unwrap();
    let mut roll = HardwareRoll::new().unwrap();

    pitch.set_target_step(600);
    roll.set_target_step(-600);

    let mut pitch_max_seen: f64 = 0.0;
    let mut roll_max_seen: f64 = 0.0;

    for _ in 0..600 {
        step_pitch(&mut pitch_ramp, &mut pitch);
        step_roll(&mut roll_ramp, &mut roll);

        pitch_max_seen = pitch_max_seen.max(pitch_ramp.speed);
        roll_max_seen = roll_max_seen.max(roll_ramp.speed);

        assert!(pitch_ramp.speed <= PITCH_MAX_PULSES_PER_SECOND);
        assert!(roll_ramp.speed <= ROLL_MAX_PULSES_PER_SECOND);
    }

    assert!(!pitch.is_moving());
    assert!(!roll.is_moving());

    assert!(
        (pitch_max_seen - PITCH_MAX_PULSES_PER_SECOND).abs()
            < 1e-6
    );

    assert!(
        (roll_max_seen - ROLL_MAX_PULSES_PER_SECOND).abs()
            < 1e-6
    );
}
