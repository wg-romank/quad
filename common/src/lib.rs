#![cfg_attr(not(feature = "godot"), no_std)]

#[derive(Debug)]
pub struct SpatialOrientation {
    pub pitch: f32,
    pub roll: f32,
}

impl SpatialOrientation {
    pub fn compute_corrections(&self, desired: &SpatialOrientation) -> [f32; 4] {
        // axes swapped
        // pitch goes from - to =
        // roll goes from + to -
        let e_p = (desired.pitch - self.pitch) / PI;
        let e_r = (desired.roll - (- self.roll)) / PI;

        let delta_x1 = e_p + e_r;
        let delta_x2 = e_p - e_r;
        let delta_x3 = -e_p - e_r;
        let delta_x4 = -e_p + e_r;

        [
            delta_x1, delta_x2, delta_x3, delta_x4
        ]
    }
}

#[derive(Debug)]
pub struct QuadState {
    pub throttle: f32,
    pub led: bool,
    pub stabilisation: bool,
    pub desired_orientation: SpatialOrientation,
    pub mode: MotorsMode,
    // todo:
    // orientation: SpatialOrientation
}

impl Default for QuadState {
    fn default() -> Self {
        Self {
            throttle: 0.0,
            led: false,
            stabilisation: false,
            desired_orientation: SpatialOrientation { pitch: 0., roll: 0. },
            mode: MotorsMode::All,
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
            Commands::SwitchMode(m) =>
                self.mode = m,
        }
    }
}

use core::f32::consts::PI;

pub use serde::{Serialize, Deserialize};
pub use postcard;
pub use heapless;
#[cfg(feature = "godot")]
pub use gdnative;
#[cfg(feature = "godot")]
use gdnative::prelude::{ToVariant, FromVariant};

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "godot", derive(ToVariant, FromVariant))]
pub enum MotorsMode {
    All, X1, X2, X3, X4
}

impl From<u32> for  MotorsMode {
    fn from(v: u32) -> Self {
        match v {
            0 => Self::All,
            1 => Self::X1,
            2 => Self::X2,
            3 => Self::X3,
            4 => Self::X4,
            _ => panic!("wrong value {}, expected range 0-4", v)
        }
    }
}


#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "godot", derive(ToVariant, FromVariant))]
pub enum Commands {
    Throttle(f32),
    Stabilisation(bool),
    Led(bool),
    Angles(f32, f32),
    SwitchMode(MotorsMode)
}

pub const COMMANDS_SIZE: usize = core::mem::size_of::<Commands>();
