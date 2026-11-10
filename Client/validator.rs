
pub fn is_valid_orientation(
    yaw: i32,
    pitch: i32,
    roll: i32,
) -> bool {
    yaw.is_finite()
        && pitch.is_finite()
        && roll.is_finite()
}
