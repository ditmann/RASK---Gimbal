
use super::RotationMath::{
    degrees_to_steps,
    matrix_to_yaw_pitch_roll,
};

fn check_rotation(
    matrix: [[f32; 4]; 3],
    expected_degrees: [f32; 3],
    expected_steps: [i32; 3],
) {
    let degrees = matrix_to_yaw_pitch_roll(matrix);

    for i in 0..3 {
        assert!(
            (degrees[i] - expected_degrees[i]).abs() < 0.001,
            "Axis {i}: expected {} degrees, got {}",
            expected_degrees[i],
            degrees[i]
        );
    }

    let steps = degrees_to_steps(degrees);

    assert_eq!(steps, expected_steps);
}

#[test]
fn test_no_rotation() {
    let matrix = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
    ];

    check_rotation(
        matrix,
        [0.0, 0.0, 0.0],
        [0, 0, 0],
    );
}

#[test]
fn test_positive_yaw() {
    let matrix = [
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [-1.0, 0.0, 0.0, 0.0],
    ];

    check_rotation(
        matrix,
        [90.0, 0.0, 0.0],
        [300, 0, 0],
    );
}

#[test]
fn test_negative_yaw() {
    let matrix = [
        [0.0, 0.0, -1.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [1.0, 0.0, 0.0, 0.0],
    ];

    check_rotation(
        matrix,
        [-90.0, 0.0, 0.0],
        [-300, 0, 0],
    );
}

#[test]
fn test_pitch_rotation() {
    let angle = 30.0_f32.to_radians();

    let matrix = [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, angle.cos(), angle.sin(), 0.0],
        [0.0, -angle.sin(), angle.cos(), 0.0],
    ];

    check_rotation(
        matrix,
        [0.0, 30.0, 0.0],
        [0, 33, 0],
    );
}

#[test]
fn test_negative_roll() {
    let angle = (-30.0_f32).to_radians();

    let matrix = [
        [angle.cos(), -angle.sin(), 0.0, 0.0],
        [angle.sin(), angle.cos(), 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
    ];

    check_rotation(
        matrix,
        [0.0, 0.0, -30.0],
        [0, 0, -33],
    );
}

#[test]
fn test_step_rounding() {
    let steps = degrees_to_steps(
        vec![0.3, 0.3, -0.3]
    );

    assert_eq!(steps, vec![1, 0, 0]);
}
