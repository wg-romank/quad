use bluetooth_serial_port::*;
use gdnative::prelude::*;
use std::io::Read;

use std::panic;

use common::EOT;
use common::SpatialOrientation;

pub const CHUNK_SIZE: usize = 16;
pub const BUF_SIZE: usize = 40;

#[derive(NativeClass)]
#[inherit(Node)]
pub struct Sensor {
    socket: Option<BtSocket>,
    buf: [u8; BUF_SIZE],
    chunk: [u8; CHUNK_SIZE],
    idx: usize,
    last_read_idx: usize,
    last_read: (f32, f32, f32),
}

#[methods]
impl Sensor {
    fn new(_owner: &Node) -> Self {
        Sensor {
            socket: None,
            buf: [0; BUF_SIZE],
            chunk: [0; CHUNK_SIZE],
            idx: 0,
            last_read_idx: 0,
            last_read: (0.0, 0.0, 0.0),
        }
    }

    #[export]
    fn connect(&mut self, _owner: &Node, sensor_mac: String) {
        let mut socket = BtSocket::new(BtProtocol::RFCOMM).unwrap();
        let mac_raw = hex::decode(sensor_mac).unwrap();
        let mut mac: [u8; 6] = [0; 6];
        mac.copy_from_slice(&mac_raw);

        godot_print!("connection {:?}", socket.connect(BtAddr(mac)));

        self.socket = Some(socket);
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
