
use crate::motor_driver::HardwareYaw;
use crate::orientation::Orientation;

pub struct GimbalController {
    target: Option<Orientation>,
    yaw_motor: HardwareYaw,
}

impl GimbalController {
    pub fn new() -> Self {
        Self {
            target: None,

            yaw_motor: HardwareYaw::new()
                .expect("Failed to initialize yaw GPIO"),
        }
    }

    pub fn set_target(&mut self, orientation: Orientation) {
        self.target = Some(orientation);

        // Only the yaw motor is connected for now
        self.yaw_motor.set_target_angle(orientation.yaw);
    }

    pub fn update(&mut self, dt_seconds: f32) {
        self.yaw_motor.update(dt_seconds);
    }

    pub fn clear_target(&mut self) {
        self.target = None;
        self.yaw_motor.stop();

        println!("Yaw target cleared.");
    }

    pub fn print_status(&self) {
        println!("{}", self.yaw_motor.status());
    }
}
