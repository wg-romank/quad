#![no_std]

#[derive(Debug)]
pub struct SpatialOrientation {
    pub pitch: f32,
    pub roll: f32,
}

impl SpatialOrientation {
    fn compute_corrections(&self, desired: SpatialOrientation) -> [f32; 4] {
        [0., 0., 0., 0.]
    }
}

#[derive(Debug)]
pub struct QuadState {
    pub throttle: f32,
    pub led: bool,
    pub stabilisation: bool,
    pub desired_orientation: SpatialOrientation,
    // todo:
    // orientation: SpatialOrientation
}

impl Default for QuadState {
    fn default() -> Self {
        Self {
            throttle: 0.0,
            led: false,
            stabilisation: false,
            desired_orientation: SpatialOrientation { pitch: 0., roll: 0. }
        }
    }
}

impl QuadState {
    pub fn update(&mut self, command: Commands) {
        match command {
            Commands::Throttle(t) =>
                self.throttle = t,
            Commands::Led(on) =>
                self.led = on,
            Commands::Stabilisation(on) =>
                self.stabilisation = on,
            Commands::Angles(p, r) =>
                self.desired_orientation = SpatialOrientation { pitch: p, roll: r },
        }
    }
}

pub use serde::{Serialize, Deserialize};
pub use postcard;
pub use heapless;

#[derive(Serialize, Deserialize)]
pub enum Commands {
    Throttle(f32),
    Stabilisation(bool),
    Led(bool),
    Angles(f32, f32)
}

pub const COMMANDS_SIZE: usize = core::mem::size_of::<Commands>();
