use std::sync::{Arc, Mutex};

use crate::musical_time::time::{Clock, Transport};
use crate::ugens::core::{Aug, Proc};

use crate::audiodevice::AudioDevice;

pub struct SoundSystem {
    transport: Arc<Mutex<Transport>>,
    root_ug: Aug,
    lock: Arc<Mutex<bool>>,
}

impl SoundSystem {
    pub fn new(transport: Arc<Mutex<Transport>>, ug: Aug, lock: Arc<Mutex<bool>>) -> SoundSystem {
        SoundSystem {
            transport: transport,
            root_ug: ug,
            lock: lock,
        }
    }

    pub fn run(&mut self, ad: &AudioDevice) {
        ad.run(|mut buffer| {
            let mut iter = buffer.iter_mut();
            loop {
                let (mut l, mut r) = (0.0, 0.0);
                if let Ok(_) = self.lock.lock() {
                    let mut transport = self.transport.lock().unwrap();
                    let s = self.root_ug.0.lock().unwrap().proc(&transport);
                    l = s.0;
                    r = s.1;
                    transport.inc();
                }

                match iter.next() {
                    Some(lref) => *lref = l as f32,
                    None => break,
                }
                match iter.next() {
                    Some(rref) => *rref = r as f32,
                    None => break,
                }
            }
        });
    }
}
