use crate::gimbal_controller::GimbalController;
use crate::orientation::Orientation;

pub enum MovementCommand {
    SetTarget(Orientation),
    Stop,
}

pub fn handle_orientation(
    controller: &mut GimbalController,
    orientation: Orientation,
) {
    controller.set_target(orientation);
}
