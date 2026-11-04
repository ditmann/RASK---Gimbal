

pub struct MatrixToDegrees {
}
// input matrix get yaw, pitch and roll

impl MatrixToDegrees {
    pub fn matrix_to_yaw_pitch_roll(matrix:[[f32; 4]; 3]) -> Vec<f32> {
        let value00 = matrix[0][0];
        let value10 = matrix[1][0];
        let value20 = matrix[2][0];
        let value11 = matrix[1][1];
        let value21 = matrix[2][1];

        println!("value00: {value00}");
        println!("value00: {value10}");
        println!("value10: {value20}");
        println!("value11: {value11}");
        println!("value21: {value21}");

        let roll = (value10).asin().to_degrees();
        let yaw = (-value20).atan2(value00).to_degrees();
        let pitch = (-value21).atan2(value11).to_degrees();

        println!("yaw: {yaw}");
        println!("pitch: {pitch}");
        println!("roll: {roll}");

        vec![yaw,pitch,roll]
    }
}