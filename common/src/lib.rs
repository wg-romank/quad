#![no_std]

pub const EOT: u8 = 0b11111111;
pub const BUFF_SIZE: usize = 8;


#[derive(Debug)]
pub struct SpatialOrientation {
    pub pitch: f32,
    pub roll: f32,
}

impl SpatialOrientation {
    pub fn to_byte_array(&self) -> [u8; 8] {
        let mut result: [u8; 8] = [0; 8];
        let (one, two) = result.split_at_mut(4);
        one.copy_from_slice(&self.pitch.to_le_bytes());
        two.copy_from_slice(&self.roll.to_le_bytes());
        result
    }

    pub fn from_byte_slice(buf: &[u8]) -> SpatialOrientation {
        let (one, two) = buf.split_at(4);

        let pitch = f32::from_le_bytes(one.try_into().unwrap());
        let roll = f32::from_le_bytes(two.try_into().unwrap());

        SpatialOrientation { pitch, roll }
    }
}