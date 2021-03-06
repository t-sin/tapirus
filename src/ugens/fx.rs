use std::collections::VecDeque;

use crate::musical_time::time::Transport;
use crate::tapirlisp::types::Env;

use super::core::{
    Aug, Dump, Operate, OperateError, Proc, Signal, Slot, UGen, UgNode, Value, Walk, UG,
};

pub struct LPFilter {
    inbuf: [Signal; 2],
    outbuf: [Signal; 2],
    freq: Aug,
    q: Aug,
    src: Aug,
}

impl LPFilter {
    pub fn new(freq: Aug, q: Aug, src: Aug) -> Aug {
        Aug::new(UGen::new(UG::Proc(Box::new(LPFilter {
            inbuf: [(0.0, 0.0), (0.0, 0.0)],
            outbuf: [(0.0, 0.0), (0.0, 0.0)],
            freq: freq,
            q: q,
            src: src,
        }))))
    }
}

impl Walk for LPFilter {
    fn walk(&self, f: &mut dyn FnMut(&Aug) -> bool) {
        if f(&self.freq) {
            self.freq.walk(f);
        }
        if f(&self.q) {
            self.q.walk(f);
        }
        if f(&self.src) {
            self.src.walk(f);
        }
    }
}

impl Dump for LPFilter {
    fn dump(&self, shared_ug: &Vec<Aug>) -> UgNode {
        let mut slots = Vec::new();

        slots.push(Slot {
            ug: self.freq.clone(),
            name: "freq".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.freq) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.freq.clone()),
            },
        });
        slots.push(Slot {
            ug: self.q.clone(),
            name: "q".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.q) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.q.clone()),
            },
        });
        slots.push(Slot {
            ug: self.src.clone(),
            name: "src".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.src) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.src.clone()),
            },
        });

        UgNode::Ug("lpf".to_string(), slots)
    }
}

impl Operate for LPFilter {
    fn get(&self, pname: &str) -> Result<Aug, OperateError> {
        match pname {
            "freq" => Ok(self.freq.clone()),
            "q" => Ok(self.q.clone()),
            "src" => Ok(self.src.clone()),
            _ => Err(OperateError::ParamNotFound(format!("lpf/{}", pname))),
        }
    }

    fn get_str(&self, pname: &str) -> Result<String, OperateError> {
        match self.get(pname) {
            Ok(aug) => {
                if let Some(v) = aug.to_val() {
                    Ok(v.to_string())
                } else {
                    Err(OperateError::CannotRepresentAsString(format!(
                        "lpf/{}",
                        pname
                    )))
                }
            }
            Err(err) => Err(err),
        }
    }

    fn set(&mut self, pname: &str, ug: Aug) -> Result<bool, OperateError> {
        match pname {
            "freq" => {
                self.freq = ug;
                Ok(true)
            }
            "q" => {
                self.q = ug;
                Ok(true)
            }
            "src" => {
                self.src = ug;
                Ok(true)
            }
            _ => Err(OperateError::ParamNotFound(format!("lpf/{}", pname))),
        }
    }

    fn set_str(&mut self, pname: &str, data: String) -> Result<bool, OperateError> {
        let mut data = data.clone();
        data.retain(|c| c != '\n' && c != ' ');

        match pname {
            "freq" => {
                if let Ok(v) = data.parse::<f64>() {
                    self.freq = Aug::val(v);
                    Ok(true)
                } else {
                    let err =
                        OperateError::CannotParseNumber(format!("lpf/{}", pname), data.clone());
                    Err(err)
                }
            }
            "q" => {
                if let Ok(v) = data.parse::<f64>() {
                    self.q = Aug::val(v);
                    Ok(true)
                } else {
                    let err =
                        OperateError::CannotParseNumber(format!("lpf/{}", pname), data.clone());
                    Err(err)
                }
            }
            "src" => {
                if let Ok(v) = data.parse::<f64>() {
                    self.src = Aug::val(v);
                    Ok(true)
                } else {
                    let err =
                        OperateError::CannotParseNumber(format!("lpf/{}", pname), data.clone());
                    Err(err)
                }
            }
            _ => Err(OperateError::ParamNotFound(format!("lpf/{}", pname))),
        }
    }

    fn clear(&mut self, pname: &str) {
        match pname {
            "freq" => {
                let _ = self.set(pname, Aug::val(0.0));
            }
            "q" => {
                let _ = self.set(pname, Aug::val(0.0));
            }
            "src" => {
                let _ = self.set(pname, Aug::val(0.0));
            }
            _ => (),
        };
    }
}

impl Proc for LPFilter {
    fn proc(&mut self, transport: &Transport) -> Signal {
        let f = self.freq.proc(transport).0;
        let q = self.q.proc(transport).0;
        let (sl, sr) = self.src.proc(transport);

        let w = (2.0 * std::f64::consts::PI * f) / transport.sample_rate as f64;
        let (sw, cw) = (w.sin(), w.cos());
        let a = sw / (2.0 * q);
        let (b0, b1, b2) = ((1.0 - cw) / 2.0, 1.0 - cw, (1.0 - cw) / 2.0);
        let (a0, a1, a2) = (1.0 + a, -2.0 * cw, 1.0 - a);

        let filter = |v, in0, in1, out0, out1| {
            (b0 / a0 * v) + (b1 / a0 * in0) + (b2 / a0 * in1) - (a1 / a0 * out0) - (a2 / a0 * out1)
        };

        let l = filter(
            sl,
            self.inbuf[0].0,
            self.inbuf[1].0,
            self.outbuf[0].0,
            self.outbuf[1].0,
        );
        let r = filter(
            sr,
            self.inbuf[0].1,
            self.inbuf[1].1,
            self.outbuf[0].1,
            self.outbuf[1].1,
        );

        self.inbuf[1] = self.inbuf[0];
        self.inbuf[0] = (sl, sr);
        self.outbuf[1] = self.outbuf[0];
        self.outbuf[0] = (l, r);

        (l, r)
    }
}

pub struct Delay {
    buffer: VecDeque<Box<Signal>>,
    time: Aug,
    feedback: Aug,
    mix: Aug,
    src: Aug,
}

impl Delay {
    pub fn new(time: Aug, feedback: Aug, mix: Aug, src: Aug, env: &Env) -> Aug {
        let len = (env.transport.sample_rate * 2) as usize;
        let mut buffer = VecDeque::with_capacity(len);
        for _n in 0..len {
            buffer.push_back(Box::new((0.0, 0.0)));
        }
        Aug::new(UGen::new(UG::Proc(Box::new(Delay {
            buffer: buffer,
            time: time,
            feedback: feedback,
            mix: mix,
            src: src,
        }))))
    }
}

impl Walk for Delay {
    fn walk(&self, f: &mut dyn FnMut(&Aug) -> bool) {
        if f(&self.time) {
            self.time.walk(f);
        }
        if f(&self.feedback) {
            self.feedback.walk(f);
        }
        if f(&self.mix) {
            self.mix.walk(f);
        }
        if f(&self.src) {
            self.src.walk(f);
        }
    }
}

impl Dump for Delay {
    fn dump(&self, shared_ug: &Vec<Aug>) -> UgNode {
        let mut slots = Vec::new();

        slots.push(Slot {
            ug: self.time.clone(),
            name: "time".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.time) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.time.clone()),
            },
        });
        slots.push(Slot {
            ug: self.feedback.clone(),
            name: "feedback".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.feedback) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.feedback.clone()),
            },
        });
        slots.push(Slot {
            ug: self.mix.clone(),
            name: "mix".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.mix) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.mix.clone()),
            },
        });
        slots.push(Slot {
            ug: self.src.clone(),
            name: "src".to_string(),
            value: match shared_ug.iter().position(|e| *e == self.src) {
                Some(n) => Value::Shared(n, shared_ug.iter().nth(n).unwrap().clone()),
                None => Value::Ug(self.src.clone()),
            },
        });

        UgNode::Ug("delay".to_string(), slots)
    }
}

impl Operate for Delay {
    fn get(&self, pname: &str) -> Result<Aug, OperateError> {
        match pname {
            "time" => Ok(self.time.clone()),
            "feedback" => Ok(self.feedback.clone()),
            "mix" => Ok(self.mix.clone()),
            "src" => Ok(self.src.clone()),
            _ => Err(OperateError::ParamNotFound(format!("delay/{}", pname))),
        }
    }

    fn get_str(&self, pname: &str) -> Result<String, OperateError> {
        match self.get(pname) {
            Ok(aug) => {
                if let Some(v) = aug.to_val() {
                    Ok(v.to_string())
                } else {
                    Err(OperateError::CannotRepresentAsString(format!(
                        "delay/{}",
                        pname
                    )))
                }
            }
            Err(err) => Err(err),
        }
    }

    fn set(&mut self, pname: &str, ug: Aug) -> Result<bool, OperateError> {
        match pname {
            "time" => {
                self.time = ug;
                Ok(true)
            }
            "feedback" => {
                self.feedback = ug;
                Ok(true)
            }
            "mix" => {
                self.mix = ug;
                Ok(true)
            }
            "src" => {
                self.src = ug;
                Ok(true)
            }
            _ => Err(OperateError::ParamNotFound(format!("delay/{}", pname))),
        }
    }

    fn set_str(&mut self, pname: &str, data: String) -> Result<bool, OperateError> {
        let mut data = data.clone();
        data.retain(|c| c != '\n' && c != ' ');

        match pname {
            "time" => {
                if let Ok(v) = data.parse::<f64>() {
                    self.time = Aug::val(v);
                    Ok(true)
                } else {
                    let err =
                        OperateError::CannotParseNumber(format!("delay/{}", pname), data.clone());
                    Err(err)
                }
            }
            "feedback" => {
                if let Ok(v) = data.parse::<f64>() {
                    self.feedback = Aug::val(v);
                    Ok(true)
                } else {
                    let err =
                        OperateError::CannotParseNumber(format!("delay/{}", pname), data.clone());
                    Err(err)
                }
            }
            "mix" => {
                if let Ok(v) = data.parse::<f64>() {
                    self.mix = Aug::val(v);
                    Ok(true)
                } else {
                    let err =
                        OperateError::CannotParseNumber(format!("delay/{}", pname), data.clone());
                    Err(err)
                }
            }
            "src" => {
                if let Ok(v) = data.parse::<f64>() {
                    self.src = Aug::val(v);
                    Ok(true)
                } else {
                    let err =
                        OperateError::CannotParseNumber(format!("delay/{}", pname), data.clone());
                    Err(err)
                }
            }
            _ => Err(OperateError::ParamNotFound(format!("delay/{}", pname))),
        }
    }

    fn clear(&mut self, pname: &str) {
        match pname {
            "time" => {
                let _ = self.set(pname, Aug::val(0.0));
            }
            "feedback" => {
                let _ = self.set(pname, Aug::val(0.0));
            }
            "mix" => {
                let _ = self.set(pname, Aug::val(0.0));
            }
            "src" => {
                let _ = self.set(pname, Aug::val(0.0));
            }
            _ => (),
        };
    }
}

// TODO: factor out; same function is in `sequencer.rs`
fn sec_to_sample_num(sec: f64, transport: &Transport) -> u64 {
    (transport.sample_rate as f64 * sec) as u64
}

impl Proc for Delay {
    fn proc(&mut self, transport: &Transport) -> Signal {
        self.buffer.pop_back();
        let sig = self.src.proc(transport);
        self.buffer.push_front(Box::new(sig));
        let dtime = self.time.proc(transport).0;
        let dt = sec_to_sample_num(dtime, transport);
        let fb = self.feedback.proc(transport).0;
        let mix = self.mix.proc(transport).0;

        let (mut dl, mut dr) = (0.0, 0.0);
        let mut n = 1;
        while dt != 0 && n * dt < self.buffer.len() as u64 {
            let (l, r) = **self.buffer.get((n * dt) as usize).unwrap();
            let fbr = fb.powi(n as i32);
            dl += l * fbr;
            dr += r * fbr;
            n += 1;
        }

        (sig.0 + dl * mix, sig.1 + dr * mix)
    }
}
