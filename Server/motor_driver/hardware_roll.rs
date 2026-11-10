
use rppal::gpio::{Gpio, OutputPin};
use std::thread;
use std::time::Duration;

// Roll GPIO pins (BCM numbering)
const STEP_PIN: u8 = 22;
const DIR_PIN: u8 = 23;
// Must match DM542T microstepping settings
const STEPS_PER_REV: i32 = 400;

const PULSE_HIGH_TIME: Duration =
    Duration::from_micros(100);

const DIRECTION_SETUP_TIME: Duration =
    Duration::from_micros(100);

pub struct HardwareRoll {
    pul: OutputPin,
    dir: OutputPin,

    current_steps: i32,
    target_steps: i32,

    previous_direction: Option<bool>,
}

impl HardwareRoll {
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
        self.target_steps = steps;
    }

    pub fn is_moving(&self) -> bool {
        self.current_steps != self.target_steps
    }

    pub fn update(&mut self, _dt_seconds: f32) {
        let difference =
            self.target_steps - self.current_steps;

        if difference == 0 {
            return;
        }

        let forward = difference > 0;

        if self.previous_direction != Some(forward) {
            if forward {
                self.dir.set_high();
            } else {
                self.dir.set_low();
            }

            thread::sleep(DIRECTION_SETUP_TIME);
            self.previous_direction = Some(forward);
        }

        // One STEP pulse
        self.pul.set_high();
        thread::sleep(PULSE_HIGH_TIME);
        self.pul.set_low();

        if forward {
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
            "Roll: {} steps (target {})",
            self.current_steps,
            self.target_steps
        )
    }
}
