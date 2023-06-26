use nalgebra::Vector2;
use nalgebra::Vector3;

use common::SpatialOrientation;

pub const GYRO_FREQUENCY_HZ: u32 = 50;
pub const GYRO_DT: f32 = 1.0 / GYRO_FREQUENCY_HZ as f32;

pub trait SpatialOrientationDevice {
    fn new(acc: Vector2<f32>) -> SpatialOrientation;
    fn adjust(&mut self, gyro: Vector3<f32>, acc: Vector2<f32>);
}

impl SpatialOrientationDevice for SpatialOrientation {
    fn new(acc: Vector2<f32>) -> SpatialOrientation {
        SpatialOrientation { pitch: acc[0], roll: acc[1] }
    }

    fn adjust(&mut self, gyro: Vector3<f32>, acc: Vector2<f32>) {
        let new_pitch = self.pitch + gyro.x * GYRO_DT;
        let new_roll = self.roll + gyro.y * GYRO_DT;

        // new_pitch += self.roll * libm::sinf(gyro.z * GYRO_DT);
        // new_roll -= self.pitch * libm::sinf(gyro.z * GYRO_DT);

        self.pitch = new_pitch * 0.96 + acc[0] * 0.04;
        self.roll = new_roll * 0.96 + acc[1] * 0.04;
    }
}