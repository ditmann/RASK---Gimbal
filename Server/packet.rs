use crate::orientation::Orientation;

pub struct OrientationPacket {
    pub millis: u64,
    pub orientation: Orientation,
}

impl OrientationPacket {
    pub fn decode(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != 20 {
            return None;
        }

        let millis = u64::from_le_bytes(
            bytes[0..8].try_into().ok()?
        );

        let yaw = f32::from_le_bytes(
            bytes[8..12].try_into().ok()?
        );

        let pitch = f32::from_le_bytes(
            bytes[12..16].try_into().ok()?
        );

        let roll = f32::from_le_bytes(
            bytes[16..20].try_into().ok()?
        );

        if !yaw.is_finite()
            || !pitch.is_finite()
            || !roll.is_finite()
        {
            return None;
        }

        Some(Self {
            millis,
            orientation: Orientation {
                yaw,
                pitch,
                roll,
            },
        })
    }
}
