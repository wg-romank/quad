use nalgebra::Vector2;
use nalgebra::Vector3;

pub const GYRO_FREQUENCY_HZ: u32 = 250;
pub const GYRO_DT: f32 = 1.0 / GYRO_FREQUENCY_HZ as f32;

#[derive(Debug)]
pub struct SpatialOrientation {
    pitch: f32,
    roll: f32,
}

impl SpatialOrientation {
    pub fn new(acc: Vector2<f32>) -> SpatialOrientation {
        SpatialOrientation { pitch: acc[0], roll: acc[1] }
    }

    pub fn adjust(&mut self, gyro: Vector3<f32>, acc: Vector2<f32>) {
        let mut new_pitch = self.pitch + gyro.x * GYRO_DT;
        let mut new_roll = self.roll + gyro.y * GYRO_DT;

        // new_pitch += self.roll * libm::sinf(gyro.z * GYRO_DT);
        // new_roll -= self.pitch * libm::sinf(gyro.z * GYRO_DT);

        self.pitch = new_pitch * 0.96 + acc[0] * 0.04;
        self.roll = new_roll * 0.96 + acc[1] * 0.04;
    }

    pub fn to_byte_array(&self) -> [u8; 8] {
        let mut result: [u8; 8] = [0; 8];
        let (one, two) = result.split_at_mut(4);
        one.copy_from_slice(&self.pitch.to_le_bytes());
        two.copy_from_slice(&self.roll.to_le_bytes());
        result
    }
}