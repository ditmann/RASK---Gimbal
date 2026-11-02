
use crate::motor_driver::HardwareYaw;
use crate::orientation::Orientation;

pub struct GimbalController {
    target: Option<Orientation>,
    yaw_motor: HardwareYaw,
}

impl GimbalController {
    pub fn new() -> Self {
        let yaw_motor = HardwareYaw::new()
            .expect("Failed to initialize Raspberry Pi GPIO");

        Self {
            target: None,
            yaw_motor,
        }
    }

    pub fn set_target(&mut self, orientation: Orientation) {
        self.target = Some(orientation);

        // Only yaw is physically connected for now
        self.yaw_motor.set_target_angle(orientation.yaw);
    }

    pub fn update(&mut self, dt_seconds: f32) {
        self.yaw_motor.update(dt_seconds);
    }

    pub fn is_moving(&self) -> bool {
        self.yaw_motor.is_moving()
    }

    pub fn clear_target(&mut self) {
        self.target = None;
        self.yaw_motor.stop();
    }

    pub fn print_status(&self) {
        println!("{}", self.yaw_motor.status());
    }
}
