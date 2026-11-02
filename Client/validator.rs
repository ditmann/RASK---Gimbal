pub fn is_valid_orientation(
    yaw: f32,
    pitch: f32,
    roll: f32,
) -> bool {
    yaw.is_finite()
        && pitch.is_finite()
        && roll.is_finite()
}
