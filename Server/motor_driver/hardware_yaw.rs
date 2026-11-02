
use rppal::gpio::{Gpio, OutputPin};
use std::thread;
use std::time::Duration;

const STEPS_PER_REV: f32 = 400.0;

// Initial bench-test limit: approximately +/- 4.5 degrees
const MAX_TEST_STEPS: i64 = 5;

const STEP_DELAY: Duration = Duration::from_millis(2);

pub struct HardwareYaw {
    pul: OutputPin,
    dir: OutputPin,

    current_steps: i64,
    target_steps: i64,
}

impl HardwareYaw {
    pub fn new() -> Result<Self, rppal::gpio::Error> {
        let gpio = Gpio::new()?;

        let pul = gpio.get(17)?.into_output_low();
        let dir = gpio.get(27)?.into_output_low();

        Ok(Self {
            pul,
            dir,
            current_steps: 0,
            target_steps: 0,
        })
    }

    pub fn set_target_angle(&mut self, angle: f32) {
        if !angle.is_finite() {
            return;                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                              
        }

        // Convert degrees into absolute motor steps
        let steps = ((angle / 360.0) * STEPS_PER_REV)
            .round() as i64;

        // Restrict movement during first hardware tests
        self.target_steps = steps.clamp(
            -MAX_TEST_STEPS,
            MAX_TEST_STEPS,
        );
    }

    pub fn update(&mut self, _dt_seconds: f32) {
        let difference =
            self.target_steps - self.current_steps;

        if difference == 0 {
            return;
        }

        if difference > 0 {
            self.dir.set_high();
        } else {
            self.dir.set_low();
        }

        // Allow direction signal to settle
        thread::sleep(Duration::from_micros(100));

        // Generate one STEP pulse
        self.pul.set_high();
        thread::sleep(STEP_DELAY);

        self.pul.set_low();
        thread::sleep(STEP_DELAY);

        // Update estimated position
        if difference > 0 {
            self.current_steps += 1;
        } else {
            self.current_steps -= 1;
        }
    }

    pub fn stop(&mut self) {
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
