use std::cmp::{Eq, PartialEq};
use std::hash::{Hash, Hasher};
use std::sync::{Arc, Mutex};

use crate::musical_time::event::Message;
use crate::musical_time::time::{Measure, Transport};
use crate::musical_time::utils::{to_len, to_note, to_pos, to_str};

//// types and traits

pub trait Walk {
    fn walk(&self, f: &mut dyn FnMut(&Aug) -> bool);
}

type OpName = String;

#[derive(Clone)]
pub enum Value {
    Number(f64),
    Table(Vec<f64>),
    Pattern(Vec<String>),
    Ug(Aug),
    Shared(usize, Aug),
}

pub struct Slot {
    pub name: String,
    pub value: Value,
    pub ug: Aug,
}

pub enum UgNode {
    Val(Value),
    Ug(OpName, Vec<Slot>),
    UgRest(OpName, Vec<Slot>, String, Vec<Box<Value>>),
}

pub trait Dump: Walk {
    fn dump(&self, shared_ug: &Vec<Aug>) -> UgNode;
}

#[derive(Debug)]
pub enum OperateError {
    NotUgen,
    CannotParsePattern(String, String),
    CannotParseNumber(String, String),
    ParamNotFound(String),
    CannotRepresentAsString(String),
}

pub trait Operate: Dump {
    fn get(&self, pname: &str) -> Result<Aug, OperateError>;
    fn get_str(&self, pname: &str) -> Result<String, OperateError>;
    fn set(&mut self, pname: &str, ug: Aug) -> Result<bool, OperateError>;
    fn set_str(&mut self, pname: &str, data: String) -> Result<bool, OperateError>;
    fn clear(&mut self, pname: &str);
}

pub type Signal = (f64, f64);

pub trait Proc: Operate {
    fn proc(&mut self, transport: &Transport) -> Signal;
}

pub trait Osc: Proc {
    fn get_ph(&self) -> f64;
    fn set_ph(&mut self, ph: f64);
    fn get_freq(&self) -> Aug;
    fn set_freq(&mut self, freq: Aug);
}

#[derive(Clone)]
pub enum ADSR {
    Attack,
    Decay,
    Sustin,
    Release,
    None,
}

pub trait Eg: Proc {
    fn get_state(&self) -> ADSR;
    fn set_state(&mut self, state: ADSR, eplaced: u64);
}

pub struct Table(pub Arc<Mutex<Vec<f64>>>);
pub struct Pattern(pub Arc<Mutex<Vec<Box<Message>>>>);

pub enum UG {
    Val(f64),
    Proc(Box<dyn Proc + Send>),
    Osc(Box<dyn Osc + Send>),
    Eg(Box<dyn Eg + Send>),
    Tab(Table),
    Pat(Pattern),
}

pub struct UGen {
    pub id: usize,
    pub last_tick: u64,
    pub last_sig: Signal,
    pub ug: UG,
}

pub struct Aug(pub Arc<Mutex<UGen>>);

// trait implementations for Table

impl Table {
    pub fn new(data: Vec<f64>) -> Table {
        Table(Arc::new(Mutex::new(data)))
    }

    pub fn parse_str(data: String) -> Option<Vec<f64>> {
        let mut table = Vec::new();
        for s in data.trim().split(' ') {
            if let Ok(n) = s.parse::<f64>() {
                table.push(n);
            } else {
                return None;
            }
        }
        Some(table)
    }
}

impl Walk for Table {
    fn walk(&self, _f: &mut dyn FnMut(&Aug) -> bool) {}
}

impl Dump for Table {
    fn dump(&self, _shared_vec: &Vec<Aug>) -> UgNode {
        let mut vec = Vec::new();
        for v in self.0.lock().unwrap().iter() {
            vec.push(*v);
        }
        UgNode::Val(Value::Table(vec))
    }
}

// trait implementations for Pattern

impl Pattern {
    pub fn new(data: Vec<Box<Message>>) -> Pattern {
        Pattern(Arc::new(Mutex::new(data)))
    }

    pub fn parse_str_1(token: &str) -> Result<Message, bool> {
        match token {
            "loop" => Ok(Message::Loop),
            s => {
                let n: Vec<&str> = s.split(':').collect();
                if n.len() != 2 {
                    Err(false)
                } else {
                    if let Some(pitch) = to_note(n[0]) {
                        if let Ok(len) = n[1].parse::<u32>() {
                            Ok(Message::Note(pitch, to_pos(len)))
                        } else {
                            Err(false)
                        }
                    } else {
                        Err(false)
                    }
                }
            }
        }
    }

    pub fn parse_str(data: String) -> Result<Vec<Box<Message>>, bool> {
        let mut msgs = Vec::new();
        for token in data.trim().split(' ') {
            if let Ok(msg) = Pattern::parse_str_1(token) {
                msgs.push(Box::new(msg));
            } else {
                return Err(false);
            }
        }
        Ok(msgs)
    }
}

impl Walk for Pattern {
    fn walk(&self, _f: &mut dyn FnMut(&Aug) -> bool) {}
}

impl Dump for Pattern {
    fn dump(&self, _shared_vec: &Vec<Aug>) -> UgNode {
        let mut vec = Vec::new();
        let m = Measure { beat: 4, note: 4 };

        for ev in self.0.lock().unwrap().iter() {
            match &**ev {
                Message::Note(pitch, len) => {
                    let pitch_s = to_str(&pitch);
                    let len_s = to_len(&len, &m);
                    vec.push(format!("{}:{}", pitch_s, len_s));
                }
                Message::Loop => vec.push("loop".to_string()),
            }
        }
        UgNode::Val(Value::Pattern(vec))
    }
}

// trait implementations for UG

impl Walk for UG {
    fn walk(&self, f: &mut dyn FnMut(&Aug) -> bool) {
        match self {
            UG::Val(_) => (),
            UG::Proc(u) => u.walk(f),
            UG::Osc(u) => u.walk(f),
            UG::Eg(u) => u.walk(f),
            UG::Tab(_) => (),
            UG::Pat(_) => (),
        }
    }
}

impl Dump for UG {
    fn dump(&self, shared_vec: &Vec<Aug>) -> UgNode {
        match self {
            UG::Val(v) => UgNode::Val(Value::Number(*v)),
            UG::Proc(u) => u.dump(shared_vec),
            UG::Osc(u) => u.dump(shared_vec),
            UG::Eg(u) => u.dump(shared_vec),
            UG::Tab(t) => t.dump(shared_vec),
            UG::Pat(p) => p.dump(shared_vec),
        }
    }
}

impl Operate for UG {
    fn get(&self, _pname: &str) -> Result<Aug, OperateError> {
        Err(OperateError::NotUgen)
    }
    fn get_str(&self, _pname: &str) -> Result<String, OperateError> {
        Err(OperateError::NotUgen)
    }
    fn set(&mut self, _pname: &str, _ug: Aug) -> Result<bool, OperateError> {
        Ok(true)
    }
    fn set_str(&mut self, _pname: &str, _data: String) -> Result<bool, OperateError> {
        Ok(true)
    }
    fn clear(&mut self, _pname: &str) {}
}

impl Proc for UG {
    fn proc(&mut self, transport: &Transport) -> Signal {
        match self {
            UG::Val(v) => (*v, *v),
            UG::Proc(u) => u.proc(transport),
            UG::Osc(u) => u.proc(transport),
            UG::Eg(u) => u.proc(transport),
            UG::Tab(_) => (0.0, 0.0),
            UG::Pat(_) => (0.0, 0.0),
        }
    }
}

impl Osc for UG {
    fn set_ph(&mut self, ph: f64) {
        match self {
            UG::Osc(u) => u.set_ph(ph),
            _ => (),
        }
    }

    fn get_ph(&self) -> f64 {
        match self {
            UG::Osc(u) => u.get_ph(),
            _ => 0.0,
        }
    }

    fn set_freq(&mut self, freq: Aug) {
        match self {
            UG::Osc(u) => u.set_freq(freq),
            _ => (),
        }
    }

    fn get_freq(&self) -> Aug {
        match self {
            UG::Osc(u) => u.get_freq(),
            _ => Aug::val(0.0),
        }
    }
}

impl Eg for UG {
    fn get_state(&self) -> ADSR {
        match self {
            UG::Eg(u) => u.get_state(),
            _ => ADSR::None,
        }
    }

    fn set_state(&mut self, state: ADSR, eplaced: u64) {
        match self {
            UG::Eg(u) => u.set_state(state, eplaced),
            _ => (),
        }
    }
}

// trait implementations for UGen

impl UGen {
    pub fn new(ug: UG) -> UGen {
        UGen {
            id: 0, // FIXME
            last_tick: 0,
            last_sig: (0.0, 0.0),
            ug: ug,
        }
    }
}

impl Walk for UGen {
    fn walk(&self, f: &mut dyn FnMut(&Aug) -> bool) {
        self.ug.walk(f);
    }
}

impl Dump for UGen {
    fn dump(&self, shared_ug: &Vec<Aug>) -> UgNode {
        self.ug.dump(shared_ug)
    }
}

impl Operate for UGen {
    fn get(&self, pname: &str) -> Result<Aug, OperateError> {
        match &self.ug {
            UG::Proc(u) => u.get(pname),
            UG::Osc(u) => u.get(pname),
            UG::Eg(u) => u.get(pname),
            _ => Err(OperateError::NotUgen),
        }
    }

    fn get_str(&self, pname: &str) -> Result<String, OperateError> {
        match &self.ug {
            UG::Proc(u) => u.get_str(pname),
            UG::Osc(u) => u.get_str(pname),
            UG::Eg(u) => u.get_str(pname),
            _ => Err(OperateError::NotUgen),
        }
    }

    fn set(&mut self, pname: &str, ug: Aug) -> Result<bool, OperateError> {
        match &mut self.ug {
            UG::Proc(u) => u.set(pname, ug),
            UG::Osc(u) => u.set(pname, ug),
            UG::Eg(u) => u.set(pname, ug),
            _ => Err(OperateError::NotUgen),
        }
    }

    fn set_str(&mut self, pname: &str, data: String) -> Result<bool, OperateError> {
        match &mut self.ug {
            UG::Proc(u) => u.set_str(pname, data),
            UG::Osc(u) => u.set_str(pname, data),
            UG::Eg(u) => u.set_str(pname, data),
            _ => Err(OperateError::NotUgen),
        }
    }

    fn clear(&mut self, pname: &str) {
        match &mut self.ug {
            UG::Proc(u) => u.clear(pname),
            UG::Osc(u) => u.clear(pname),
            UG::Eg(u) => u.clear(pname),
            _ => (),
        }
    }
}

impl Proc for UGen {
    fn proc(&mut self, transport: &Transport) -> Signal {
        if self.last_tick == transport.tick {
            self.last_sig
        } else {
            self.last_tick = transport.tick;
            let sig = self.ug.proc(transport);
            self.last_sig = sig;
            sig
        }
    }
}

// trait implementations for Aug

impl Aug {
    pub fn new(ug: UGen) -> Aug {
        Aug(Arc::new(Mutex::new(ug)))
    }

    pub fn val(v: f64) -> Aug {
        Aug::new(UGen::new(UG::Val(v)))
    }

    pub fn to_val(&self) -> Option<f64> {
        match self.0.lock().unwrap().ug {
            UG::Val(v) => Some(v),
            _ => None,
        }
    }
}

impl Clone for Aug {
    fn clone(&self) -> Aug {
        Aug(self.0.clone())
    }
}

impl PartialEq for Aug {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for Aug {}

impl Hash for Aug {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::into_raw(self.0.clone()).hash(state);
    }
}

impl Walk for Aug {
    fn walk(&self, f: &mut dyn FnMut(&Aug) -> bool) {
        (*self.0.lock().unwrap()).walk(f)
    }
}

impl Dump for Aug {
    fn dump(&self, shared_ug: &Vec<Aug>) -> UgNode {
        self.0.lock().unwrap().dump(shared_ug)
    }
}

impl Operate for Aug {
    fn get(&self, pname: &str) -> Result<Aug, OperateError> {
        self.0.lock().unwrap().get(pname)
    }
    fn get_str(&self, pname: &str) -> Result<String, OperateError> {
        self.0.lock().unwrap().get_str(pname)
    }
    fn set(&mut self, pname: &str, ug: Aug) -> Result<bool, OperateError> {
        self.0.lock().unwrap().set(pname, ug)
    }
    fn set_str(&mut self, pname: &str, data: String) -> Result<bool, OperateError> {
        self.0.lock().unwrap().set_str(pname, data)
    }
    fn clear(&mut self, pname: &str) {
        self.0.lock().unwrap().clear(pname)
    }
}

impl Proc for Aug {
    fn proc(&mut self, transport: &Transport) -> Signal {
        self.0.lock().unwrap().proc(transport)
    }
}
