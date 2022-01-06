use gdnative::prelude::*;
use probe_rs::Probe;
use probe_rs_rtt::Rtt;
use std::sync::{Arc, Mutex};
use std::convert::TryInto;

#[derive(NativeClass)]
#[inherit(Node)]
pub struct Sensor {
    rtt: Option<Rtt>,
    channell: Option<probe_rs_rtt::UpChannel>,
}

#[methods]
impl Sensor {
    fn new(_owner: &Node) -> Self {
        Sensor {
            rtt: None,
            channell: None,
        }
    }

    #[export]
    fn _ready(&mut self, _owner: &Node) {
        godot_print!("hello, rworld.");
        let tmp = Probe::list_all();
        godot_print!("probes {:?}", tmp);
        let probe = tmp[0].open().expect("unable to open probe");
        godot_print!("hello, probe.");
        let session = probe
            .attach("stm32f103C8")
            .expect("unable to start session");
        godot_print!("hello, session.");
        let rtt = Rtt::attach(Arc::new(Mutex::new(session))).expect("unable to get rtt");
        godot_print!("hello, rtt.");

        self.rtt = Some(rtt);
    }

    fn try_get_channel(&mut self) {
        match &mut self.rtt {
            Some(rtt) => {
                if let Some(input) = rtt.up_channels().take(0) {
                    self.channell = Some(input);
                }
            }
            None => (),
        }
    }

    fn parse_data(ch: &mut probe_rs_rtt::UpChannel) -> (f32, f32, f32) {
        let mut buf = [0; 8];
        ch.read(&mut buf).expect("failed to read from channel");
        let (one, two) = buf.split_at(4);

        let x = f32::from_le_bytes(one.try_into().unwrap());
        let y = f32::from_le_bytes(two.try_into().unwrap());

        // (-0.5, 0.3, -0.7)
        (x, y, 0.0)
    }

    #[export]
    fn get_angles(&mut self, _owner: &Node) -> (f32, f32, f32) {
        if let Some(input) = &mut self.channell {
            // godot_print!("channel {:?}", input);
            Sensor::parse_data(input)
        } else {
            self.try_get_channel();
            (0.5, 0.3, 0.7)
        }
    }
}

fn init(handle: InitHandle) {
    handle.add_class::<Sensor>();
}

godot_init!(init);
