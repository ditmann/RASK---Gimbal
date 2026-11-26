#[cfg(all(
    target_os = "linux",
    any(target_arch = "arm", target_arch = "aarch64")
))]
mod hardware_yaw;

#[cfg(all(
    target_os = "linux",
    any(target_arch = "arm", target_arch = "aarch64")
))]
mod hardware_pitch;

#[cfg(all(
    target_os = "linux",
    any(target_arch = "arm", target_arch = "aarch64")
))]
mod hardware_roll;

#[cfg(all(
    target_os = "linux",
    any(target_arch = "arm", target_arch = "aarch64")
))]
pub use hardware_yaw::HardwareYaw;

#[cfg(all(
    target_os = "linux",
    any(target_arch = "arm", target_arch = "aarch64")
))]
pub use hardware_pitch::HardwarePitch;

#[cfg(all(
    target_os = "linux",
    any(target_arch = "arm", target_arch = "aarch64")
))]
pub use hardware_roll::HardwareRoll;

// Simulator for Windows og andre ikke-Raspberry-Pi-plattformer.
#[cfg(not(all(
    target_os = "linux",
    any(target_arch = "arm", target_arch = "aarch64")
)))]
mod simulated_stepper;

#[cfg(not(all(
    target_os = "linux",
    any(target_arch = "arm", target_arch = "aarch64")
)))]
pub use simulated_stepper::{
    HardwareYaw,
    HardwarePitch,
    HardwareRoll,
};
