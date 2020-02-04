use crate::musical_time::time::Pos;

pub type Freq = f64;

#[derive(Debug)]
pub enum Event {
    On(Pos, Freq),
    Kick(Pos),
    Off(Pos),
    Loop(Pos),
}

impl Clone for Event {
    fn clone(&self) -> Self {
        match self {
            Event::On(pos, freq) => Event::On(pos.clone(), *freq),
            Event::Kick(pos) => Event::Kick(pos.clone()),
            Event::Off(pos) => Event::Off(pos.clone()),
            Event::Loop(pos) => Event::Loop(pos.clone()),
        }
    }
}

pub type NoteNum = u32;
pub type Octave = u32;

#[derive(Debug, Clone)]
pub enum Pitch {
    Pitch(NoteNum, Octave),
    Kick,
    Rest,
}

#[derive(Debug, Clone)]
pub enum Message {
    Note(Pitch, Pos),
    Loop,
}
