
use std::convert::Infallible;

pub struct SimulatedStepper<const AXIS: u8> {
    current_step: i64,
    target_step: i64,
}

impl<const AXIS: u8> SimulatedStepper<AXIS> {
    pub fn new() -> Result<Self, Infallible> {
        Ok(Self {
            current_step: 0,
            target_step: 0,
        })
    }

    pub fn set_target_step(&mut self, target: i32) {
        self.target_step = i64::from(target);
    }

    pub fn remaining_steps(&self) -> i64 {
        self.target_step - self.current_step
    }

    pub fn is_moving(&self) -> bool {
        self.remaining_steps() != 0
    }

    pub fn step_in_direction(&mut self, forward: bool) {
        if forward {
            self.current_step += 1;
        } else {
            self.current_step -= 1;
        }
    }

    pub fn stop(&mut self) {
        self.target_step = self.current_step;
    }

    // MotorLoop sender STEP-pulsene direkte.
    // Vi skal derfor ikke generere ekstra steps her.
    pub fn update(&mut self, _dt_seconds: f32) {}

    pub fn status(&self) -> String {
        let axis = match AXIS {
            0 => "Yaw",
            1 => "Pitch",
            2 => "Roll",
            _ => "Unknown",
        };

        format!(
            "{}: {} steps (target {} steps)",
            axis,
            self.current_step,
            self.target_step
        )
    }
}

pub type HardwareYaw = SimulatedStepper<0>;
pub type HardwarePitch = SimulatedStepper<1>;
pub type HardwareRoll = SimulatedStepper<2>;


