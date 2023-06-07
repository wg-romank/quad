use bluetooth_serial_port::*;
use gdnative::prelude::*;

use hex::FromHexError;

use std::io::Read;
use std::io::Write;

use std::ops::Deref;
use std::panic;

use common::EOT;
use common::SpatialOrientation;
use common::Commands;

use common::postcard::to_vec;
use common::heapless::Vec as HVec;
use common::COMMANDS_SIZE;


pub const CHUNK_SIZE: usize = 16;
pub const BUF_SIZE: usize = 40;

#[derive(ToVariant)]
enum Stm32Error {
    BtConnection(String),
    Command(String),
    Misc(String)
}

impl From<BtError> for Stm32Error {
    fn from(e: BtError) -> Self {
        Stm32Error::BtConnection(e.to_string())
    }
}

impl From<FromHexError> for Stm32Error {
    fn from(e: FromHexError) -> Self {
        Stm32Error::Misc(format!("malformed hex string {}", e))
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

#[methods]
impl Sensor {
    fn new(_owner: &Node) -> Self {
        Self {
            socket: None,
            buf: [0; BUF_SIZE],
            chunk: [0; CHUNK_SIZE],
            idx: 0,
            last_read: (0.0, 0.0, 0.0),
        }
    }

    #[export]
    fn send_throttle(&mut self, _owner: &Node, throttle: f32) -> Result<(), Stm32Error> {
        let buf: HVec<u8, COMMANDS_SIZE> = to_vec(&Commands::Throttle(throttle)).unwrap();
        if let Some(s) = &mut self.socket {
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
    fn led(&mut self, _owner: &Node, led: bool) -> Result<(), Stm32Error> {
        let buf: HVec<u8, COMMANDS_SIZE> = to_vec(&Commands::Led(led)).unwrap();
        if let Some(s) = &mut self.socket {
            if let Err(_) = s.write(&buf.deref()) {
                Err(Stm32Error::Command(format!("unable to send led")))
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
