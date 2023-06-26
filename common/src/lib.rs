#![cfg_attr(not(feature = "godot"), no_std)]
#[cfg(feature = "godot")]
pub use gdnative;
#[cfg(feature = "godot")]
use gdnative::prelude::{ToVariant, FromVariant};

pub use serde::{Serialize, Deserialize};
pub use postcard;
pub use heapless;

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "godot", derive(ToVariant, FromVariant))]
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

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "godot", derive(ToVariant, FromVariant))]
pub struct QuadState {
    // time from RTC is in seconds
    pub last_command_time: u32,
    throttle: f32,
    pub led: bool,
    pub stabilisation: bool,
    pub desired_orientation: SpatialOrientation,
    pub mode: MotorsModeCombined,
    // todo:
    // orientation: SpatialOrientation
}

impl Default for QuadState {
    fn default() -> Self {
        Self {
            last_command_time: 0,
            throttle: 0.0,
            led: false,
            stabilisation: false,
            desired_orientation: SpatialOrientation { pitch: 0., roll: 0. },
            mode: MotorsModeCombined(0b1111),
        }
    }
}

impl QuadState {
    pub fn throttle(&self, time: u32) -> f32 {
        if self.last_command_time.abs_diff(time) <= 1 {
            self.throttle
        } else {
            0.0
        }
    }
    pub fn update(&mut self, command: Commands, time: u32) {
        self.last_command_time = time;
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
                self.mode.0 = m,
        }
    }
}

use core::f32::consts::PI;

#[repr(u8)]
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "godot", derive(ToVariant, FromVariant))]
pub enum MotorsMode {
    X1 = 0b1000,
    X2 = 0b0100,
    X3 = 0b0010,
    X4 = 0b0001
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "godot", derive(ToVariant, FromVariant))]
pub struct MotorsModeCombined(u8);

impl MotorsModeCombined {
    pub fn is_x1_enabled(&self) -> f32 {
        (self.0 & MotorsMode::X1 as u8) as f32
    }

    pub fn is_x2_enabled(&self) -> f32 {
        (self.0 & MotorsMode::X2 as u8) as f32
    }

    pub fn is_x3_enabled(&self) -> f32 {
        (self.0 & MotorsMode::X3 as u8) as f32
    }

    pub fn is_x4_enabled(&self) -> f32 {
        (self.0 & MotorsMode::X4 as u8) as f32
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "godot", derive(ToVariant, FromVariant))]
pub enum Commands {
    Throttle(f32),
    Stabilisation(bool),
    Led(bool),
    Angles(f32, f32),
    SwitchMode(u8)
}

pub const COMMANDS_SIZE: usize = core::mem::size_of::<Commands>();
