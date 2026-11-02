use crate::motor_driver::SimulatedStepper;
use crate::orientation::Orientation;

pub struct GimbalController {
    target: Option<Orientation>,

    yaw_motor: SimulatedStepper,
    pitch_motor: SimulatedStepper,
    roll_motor: SimulatedStepper,
}

impl GimbalController {
    pub fn new() -> Self {
        Self {
            target: None,

            yaw_motor: SimulatedStepper::new("Yaw"),
            pitch_motor: SimulatedStepper::new("Pitch"),
            roll_motor: SimulatedStepper::new("Roll"),
        }
    }

    pub fn set_target(&mut self, orientation: Orientation) {
        self.target = Some(orientation);

        self.yaw_motor.set_target_angle(orientation.yaw);
        self.pitch_motor.set_target_angle(orientation.pitch);
        self.roll_motor.set_target_angle(orientation.roll);
    }

    pub fn update(&mut self, dt_seconds: f32) {
        self.yaw_motor.update(dt_seconds);
        self.pitch_motor.update(dt_seconds);
        self.roll_motor.update(dt_seconds);
    }

    pub fn clear_target(&mut self) {
        self.target = None;

        self.yaw_motor.stop();
        self.pitch_motor.stop();
        self.roll_motor.stop();

        println!("Gimbal stopped. Holding simulated position.");
    }

    pub fn print_status(&self) {
        println!(
            "{} | {} | {}",
            self.yaw_motor.status(),
            self.pitch_motor.status(),
            self.roll_motor.status()
        );
    }
}
