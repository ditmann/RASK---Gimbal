
use rppal::gpio::{Gpio, OutputPin};
use std::thread;
use std::time::Duration;

// GPIO pins (BCM numbering)
const STEP_PIN: u8 = 17;
const DIR_PIN: u8 = 27;

// Must match the DM542T microstepping configuration
const STEPS_PER_REV: i32 = 400;

// STEP and DIR signal timing
const PULSE_HIGH_TIME: Duration =
    Duration::from_micros(100);

const DIRECTION_SETUP_TIME: Duration =
    Duration::from_micros(100);


pub struct HardwareYaw {
    pul: OutputPin,
    dir: OutputPin,

    current_steps: i32,
    target_steps: i32,

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


    pub fn set_target_step(&mut self, steps: i32) {
        // Target position within one revolution
        let target_step =
            (steps.rem_euclid(STEPS_PER_REV));

        // Current position within one revolution
        let current_step =
            self.current_steps.rem_euclid(STEPS_PER_REV);

        // Calculate shortest direction to target
        let mut difference =
            (target_step - current_step).rem_euclid(STEPS_PER_REV);

        if difference > STEPS_PER_REV / 2 {
            difference -= STEPS_PER_REV;
        }

        // Set target
        self.target_steps = self.current_steps + difference;
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

        // Determine movement direction
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
        // Stop at current commanded position
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
