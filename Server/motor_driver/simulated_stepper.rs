pub struct SimulatedStepper {
    axis: &'static str,
    current_angle: f32,
    target_angle: f32,
}

impl SimulatedStepper {
    pub fn new(axis: &'static str) -> Self {
        Self {
            axis,
            current_angle: 0.0,
            target_angle: 0.0,
        }
    }

    pub fn set_target_angle(&mut self, angle: f32) {
        self.target_angle = angle;
    }

    pub fn update(&mut self, dt_seconds: f32) {
        // Simulated maximum speed: 90 degrees/second
        let max_movement = 90.0 * dt_seconds;

        let difference =
            self.target_angle - self.current_angle;

        let movement = difference.clamp(
            -max_movement,
            max_movement,
        );

        self.current_angle += movement;
    }

    pub fn stop(&mut self) {
        // Hold current simulated position
        self.target_angle = self.current_angle;
    }

    pub fn status(&self) -> String {
        format!(
            "{}: {:.1}° (target {:.1}°)",
            self.axis,
            self.current_angle,
            self.target_angle
        )
    }
}
