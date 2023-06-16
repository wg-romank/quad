use bluetooth_serial_port::*;
use common::QuadState;
use common::postcard::from_bytes;
use gdnative::prelude::*;

use hex::FromHexError;

use std::io::Write;
use std::io::Read;

use std::panic;

use common::Commands;

use common::postcard::to_vec;
use common::postcard::Error as PostcardError;
use common::heapless::Vec as HVec;
use common::COMMANDS_SIZE;


pub const CHUNK_SIZE: usize = 16;
pub const BUF_SIZE: usize = 40;

#[derive(ToVariant)]
enum Stm32Error {
    BtConnection(String),
    Command(String),
    SerialisationError(String),
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

impl From<PostcardError> for Stm32Error {
    fn from(e: PostcardError) -> Self {
        Self::SerialisationError(format!("PostcardError: {:?}", e))
    }
}

#[derive(NativeClass)]
#[inherit(Node)]
pub struct Sensor {
    socket: Option<BtSocket>,
    buf: [u8; 30],
}

#[methods]
impl Sensor {
    fn new(_owner: &Node) -> Self {
        Self {
            socket: None,
            buf: [0; 30],
        }
    }

    #[export]
    fn send_command(&mut self, _owner: &Node, command: Commands) -> Result<usize, Stm32Error> {
        let buf: HVec<u8, COMMANDS_SIZE> = to_vec(&command)?;
        if let Some(s) = &mut self.socket {
            s.write(&buf)
                .map_err(|e| Stm32Error::Command(e.to_string()))
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
    fn get_angles(&mut self, _owner: &Node) -> Option<QuadState> {
        if let Some(s) = &mut self.socket {
            let read_len = s.read_exact(&mut self.buf).expect("failed to read from channel");
            let res: common::QuadState = from_bytes(&self.buf).expect("failed to deserialize");
            Some(res)
        } else {
            None
        }
    }
}

fn init(handle: InitHandle) {
    panic::set_hook(Box::new(|p| godot_print!("Panic {:?}", p)));

    handle.add_class::<Sensor>();
}

godot_init!(init);
