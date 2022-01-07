use bluetooth_serial_port::*;
use gdnative::prelude::*;
use std::convert::TryInto;
use std::io::Read;
use std::sync::{Arc, Mutex};

use std::panic;

#[derive(NativeClass)]
#[inherit(Node)]
pub struct Sensor {
    socket: Option<BtSocket>,
    buf: [u8; 100],
    idx: usize,
    last_read: (f32, f32, f32),
}

#[methods]
impl Sensor {
    fn new(_owner: &Node) -> Self {
        Sensor {
            socket: None,
            buf: [0; 100],
            idx: 0,
            last_read: (0.0, 0.0, 0.0),
        }
    }

    #[export]
    fn _ready(&mut self, _owner: &Node) {
        let mut socket = BtSocket::new(BtProtocol::RFCOMM).unwrap();
        let mac_raw = hex::decode("70F209016500").unwrap();
        let mut mac: [u8; 6] = [0; 6];
        mac.copy_from_slice(&mac_raw);

        godot_print!("connection {:?}", socket.connect(BtAddr(mac)));

        self.socket = Some(socket);
    }

    fn parse_data(buf: &[u8]) -> (f32, f32, f32) {
        let (one, two) = buf.split_at(4);

        let x = f32::from_le_bytes(one.try_into().unwrap());
        let y = f32::from_le_bytes(two.try_into().unwrap());

        (x, y, 0.0)
    }

    #[export]
    fn get_angles(&mut self, _owner: &Node) -> (f32, f32, f32) {
        if let Some(input) = &mut self.socket {
            let mut buf = [0; 40];
            let read_len = input.read(&mut buf).expect("failed to read from channel");
            // godot_print!("read {} bytes", read_len);

            if self.idx + buf.len() >= self.buf.len() {
                self.idx = 0;
            }

            self.buf[self.idx..self.idx + buf.len()].clone_from_slice(&buf);
            self.idx += read_len;

            let markers = self.buf[..self.idx]
                .iter()
                .map(|w| { if *w == 0b11111111 { 'M' } else { '.' } })
                .collect::<String>();

            // godot_print!("{}", markers);

            let m = self.buf[..self.idx]
                .split(|w| *w == 0b11111111 )
                .collect::<Vec<&[u8]>>()
                .into_iter()
                .rev()
                .next();
            
            if let Some(payload) = m {
                if payload.len() == 8 {
                    self.last_read = Sensor::parse_data(payload);
                } else {
                    // godot_print!("invalid length {}", payload.len());
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
