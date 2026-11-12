pub(crate) fn degrees_to_steps(degrees: Vec<f32>) -> Vec<i32> {
    const STEPS_PER_REV_YAW: f32 = 400.0;
    const STEPS_PER_REV_PITCH: f32 = 400.0;
    const STEPS_PER_REV_ROLL: f32 = 400.0;
    
    const YAW_GEAR_RATIO: f32 = 3.0;

    let yaw: i32 = (((degrees[0] / 360.0) * STEPS_PER_REV_YAW) * YAW_GEAR_RATIO).round() as i32;
    let pitch: i32 = ((degrees[1]  / 360.0) * STEPS_PER_REV_PITCH).round() as i32;
    let roll: i32 = ((degrees[2]  / 360.0) * STEPS_PER_REV_ROLL).round() as i32;

    println!("yaw_steps: {yaw}");
    println!("pitch_steps: {pitch}");
    println!("roll_steps: {roll}");

    vec![yaw, pitch, roll]
}
pub(crate) fn matrix_to_yaw_pitch_roll(matrix: [[f32; 4]; 3]) -> Vec<f32> {
    let value00 = matrix[0][0];
    let value10 = matrix[1][0];
    let value20 = matrix[2][0];
    let value11 = matrix[1][1];
    let value12 = matrix[1][2];


    //println!("value00: {value00}");
    //println!("value00: {value10}");
    //println!("value10: {value20}");
    //println!("value11: {value11}");
    //println!("value21: {value21}");

    let roll = (value10).asin().to_degrees();
    let yaw = (-value20).atan2(value00).to_degrees();
    let pitch = (-value12).atan2(value11).to_degrees();

    //let yaw = (-value20/value00).atan().to_degrees();
    //let pitch = (-value21/value11).atan().to_degrees();

    println!("yaw_deg: {yaw}");
    println!("pitch_deg: {pitch}");
    println!("roll_deg: {roll}");


    vec![yaw,pitch,roll]
}