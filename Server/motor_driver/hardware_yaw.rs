
use rppal::gpio::{Gpio, OutputPin};
use std::thread;
use std::time::Duration;

// GPIO pins (BCM numbering)
const STEP_PIN: u8 = 17;
const DIR_PIN: u8 = 27;

// Must match the DM332T microstepping configuration
const STEPS_PER_REV: f32 = 400.0;

// Temporary movement limit for testing
const MAX_TEST_STEPS: i64 = 1000;

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

    last_input_yaw: Option<f32>,
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
            last_input_yaw: None,
        })
    }



    pub fn set_target_angle(&mut self, angle: f32) {
        if !angle.is_finite() {
            return;
        }

        // Reject sudden VR yaw jumps
        const MAX_YAW_JUMP_DEG: f32 = 25.0;

        if let Some(previous_yaw) = self.last_input_yaw {
            let delta = (angle - previous_yaw + 180.0)
                .rem_euclid(360.0) - 180.0;

            if delta.abs() > MAX_YAW_JUMP_DEG {
                eprintln!(
                    "VR yaw jump rejected: {:.1} -> {:.1}",
                    previous_yaw,
                    angle
                );

                self.stop();
                return;
            }
        }

        self.last_input_yaw = Some(angle);

        // Calculate shortest direction around 360 degrees
        let steps_per_rev = STEPS_PER_REV.round() as i64;

        let wrapped_angle = angle.rem_euclid(360.0);

        let target_step =
            ((wrapped_angle / 360.0) * STEPS_PER_REV).round() as i64;

        let target_step =
            target_step.rem_euclid(steps_per_rev);

        let current_step =
            self.current_steps.rem_euclid(steps_per_rev);

        let mut difference =
            (target_step - current_step).rem_euclid(steps_per_rev);

        if difference > steps_per_rev / 2 {
            difference -= steps_per_rev;
        }

        // Apply the temporary test limit
        self.target_steps =
            (self.current_steps + difference)
                .clamp(-MAX_TEST_STEPS, MAX_TEST_STEPS);
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
