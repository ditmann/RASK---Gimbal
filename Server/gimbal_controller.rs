
use crate::orientation::Orientation;
use crate::motor_driver::{
    HardwareYaw,
    HardwarePitch,
    HardwareRoll,
};

pub struct GimbalController {
    target: Option<Orientation>,

    yaw_motor: HardwareYaw,
    pitch_motor: HardwarePitch,
    roll_motor: HardwareRoll,
}

impl GimbalController {
    pub fn new() -> Self {
        let yaw_motor = HardwareYaw::new()
            .expect("Failed to initialize yaw GPIO");

        let pitch_motor = HardwarePitch::new()
            .expect("Failed to initialize pitch GPIO");

        let roll_motor = HardwareRoll::new()
            .expect("Failed to initialize roll GPIO");

        Self {
            target: None,
            yaw_motor,
            pitch_motor,
            roll_motor,
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

    pub fn is_moving(&self) -> bool {
        self.yaw_motor.is_moving()
            || self.pitch_motor.is_moving()
            || self.roll_motor.is_moving()
    }

    pub fn clear_target(&mut self) {
        self.target = None;

        self.yaw_motor.stop();
        self.pitch_motor.stop();
        self.roll_motor.stop();
    }

    pub fn print_status(&self) {
        println!(
            "{} | {} | {}",
            self.yaw_motor.status(),
            self.pitch_motor.status(),
            self.roll_motor.status(),
        );
    }
}
