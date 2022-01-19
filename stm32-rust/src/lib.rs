use bluetooth_serial_port::*;
use gdnative::prelude::*;
use hex::FromHexError;
use std::io::Read;
use std::io::Write;

use std::panic;

use common::EOT;
use common::SpatialOrientation;
use common::Command;

pub const CHUNK_SIZE: usize = 16;
pub const BUF_SIZE: usize = 40;

enum Stm32Error {
    BtConnection(String),
    Command(String),
    Misc(String)
}

impl ToVariant for Stm32Error {
    fn to_variant(&self) -> Variant {
        match &self {
            &Self::BtConnection(e) => Variant::from_str(format!("failed to connect to device: {}", e)),
            &Self::Command(e) => Variant::from_str(format!("failed sending command {}", e)),
            &Self::Misc(e) => Variant::from_str(e),
        }
    }
}

impl From<BtError> for Stm32Error {
    fn from(e: BtError) -> Self {
        Stm32Error::BtConnection(e.to_string())
    }
}

impl From<FromHexError> for Stm32Error {
    fn from(_: FromHexError) -> Self {
        Stm32Error::Misc(format!("malformed hex string"))
    }
}

#[derive(NativeClass)]
#[inherit(Node)]
pub struct Sensor {
    socket: Option<BtSocket>,
    buf: [u8; BUF_SIZE],
    chunk: [u8; CHUNK_SIZE],
    idx: usize,
    last_read: (f32, f32, f32),
}

impl Drop for Sensor {
    fn drop(&mut self) {
        todo!()
    }
}

#[methods]
impl Sensor {
    fn new(_owner: &Node) -> Self {
        Sensor {
            socket: None,
            buf: [0; BUF_SIZE],
            chunk: [0; CHUNK_SIZE],
            idx: 0,
            last_read: (0.0, 0.0, 0.0),
        }
    }

    #[export]
    fn send_throttle(&mut self, _owner: &Node, throttle_on: bool, throttle: f32) -> Result<(), Stm32Error> {
        let command = Command { throttle_on, throttle };
        if let Some(s) = &mut self.socket {
            let buf = command.to_byte_array();

            if !s.write(&buf).is_ok() {
                Err(Stm32Error::Command(format!("throttle")))
            } else {
                Ok(())
            }
        } else {
            Err(Stm32Error::BtConnection(format!("not connected")))
        }
    }

    #[export]
    fn connect(&mut self, _owner: &Node, sensor_mac: String) -> Result<(), Stm32Error> {
        let mac_raw = hex::decode(sensor_mac)?;
        let mut mac: [u8; 6] = [0; 6];
        mac.copy_from_slice(&mac_raw);

        let mut socket = BtSocket::new(BtProtocol::RFCOMM)?;
        socket.connect(BtAddr(mac))?;
        self.socket = Some(socket);

        Ok(())
    }

    #[export]
    fn get_angles(&mut self, _owner: &Node) -> (f32, f32, f32) {
        if let Some(s) = &mut self.socket {
            let read_len = s.read(&mut self.chunk).expect("failed to read from channel");

            if self.idx + CHUNK_SIZE >= BUF_SIZE {
                self.idx = 0;
            }

            self.buf[self.idx..self.idx + CHUNK_SIZE].clone_from_slice(&self.chunk);
            self.idx += read_len;

            let m = self.buf[..self.idx]
                .split(|w| *w == EOT )
                .collect::<Vec<&[u8]>>()
                .into_iter()
                .rev()
                .next();
            
            if let Some(payload) = m {
                if payload.len() == common::BUFF_SIZE {
                    let so = SpatialOrientation::from_byte_slice(payload);
                    self.last_read = (so.pitch, so.roll, 0.0);
                }
            }
        }

        self.last_read
    }
}

fn init(handle: InitHandle) {
    panic::set_hook(Box::new(|p| godot_print!("Panic {:?}", p)));

    handle.add_class::<Sensor>();
}

godot_init!(init);
