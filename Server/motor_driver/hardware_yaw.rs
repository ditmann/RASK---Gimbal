
use rppal::gpio::{Gpio, OutputPin};
use std::thread;
use std::time::Duration;

// GPIO pins (BCM numbering)
const STEP_PIN: u8 = 17;
const DIR_PIN: u8 = 27;

// Must match the DM332T microstepping configuration
const STEPS_PER_REV: f32 = 400.0;

// Temporary movement limit for testing
const MAX_TEST_STEPS: i64 = 100;

// STEP and DIR signal timing
const PULSE_HIGH_TIME: Duration =
    Duration::from_micros(100);

const DIRECTION_SETUP_TIME: Duration =
    Duration::from_micros(100);

pub struct HardwareYaw {
    pul: OutputPin,
    dir: OutputPin,

    current_steps: i64,
    target_steps: i64,

    previous_direction: Option<bool>,
}

impl HardwareYaw {
    pub fn new() -> Result<Self, rppal::gpio::Error> {
        let gpio = Gpio::new()?;

        let pul = gpio.get(STEP_PIN)?.into_output_low();
        let dir = gpio.get(DIR_PIN)?.into_output_low();

        Ok(Self {
            pul,
            dir,
            current_steps: 0,
            target_steps: 0,
            previous_direction: None,
        })
    }

    pub fn set_target_angle(&mut self, angle: f32) {
        // Reject invalid values
        if !angle.is_finite() {
            return;
        }

        // Convert yaw angle to motor steps
        let steps =
            ((angle / 360.0) * STEPS_PER_REV).round() as i64;

        // Restrict movement during testing
        // self.target_steps = steps.clamp(-MAX_TEST_STEPS, MAX_TEST_STEPS, );
        self.target_steps = steps;
    }

    pub fn is_moving(&self) -> bool {
        self.current_steps != self.target_steps
    }

    pub fn update(&mut self, _dt_seconds: f32) {
        let difference =
            self.target_steps - self.current_steps;

        // Already at target
        if difference == 0 {
            return;
        }

        let forward = difference > 0;

        // Change DIR only when necessary
        if self.previous_direction != Some(forward) {
            if forward {
                self.dir.set_high();
            } else {
                self.dir.set_low();
            }

            // Allow direction signal to settle
            thread::sleep(DIRECTION_SETUP_TIME);

            self.previous_direction = Some(forward);
        }

        // Generate exactly one STEP pulse
        self.pul.set_high();
        thread::sleep(PULSE_HIGH_TIME);
        self.pul.set_low();

        // Update commanded position
        if forward {
            self.current_steps += 1;
        } else {
            self.current_steps -= 1;
        }
    }

    pub fn stop(&mut self) {
        // Stop at the current commanded position
        self.target_steps = self.current_steps;
        self.pul.set_low();
    }

    pub fn status(&self) -> String {
        format!(
            "Yaw: {} steps (target {})",
            self.current_steps,
            self.target_steps
        )
    }
}
