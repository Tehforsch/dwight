use crate::melody::{delay_after_note_ms, Note, BPM};

#[derive(Debug)]
pub enum LedState {
    On,
    Off,
}

#[derive(Debug)]
pub enum Led {
    Left,
    Right,
}

#[derive(Debug)]
pub enum RelayState {
    On,
    Off,
}

#[derive(Debug)]
pub enum Frequency {
    Some(f32),
    Silence,
}

impl Frequency {
    pub const C4: Frequency = Frequency::Some(261.63);
    pub const D4: Frequency = Frequency::Some(293.66);
    pub const E4: Frequency = Frequency::Some(329.63);
    pub const F4: Frequency = Frequency::Some(349.23);
    pub const G4: Frequency = Frequency::Some(392.00);
    pub const A4: Frequency = Frequency::Some(440.00);
    pub const B4: Frequency = Frequency::Some(493.88);
    pub const BREAK: Frequency = Frequency::Silence;
}

pub enum Switch {
    Number(usize),
    Left,
    Right,
}

pub enum SwitchState {
    Pressed,
    Released,
}

pub trait Machine {
    fn get_switch_state(&mut self, switch: Switch) -> SwitchState;
    fn set_led_state(&mut self, led: &Led, led_state: &LedState);
    fn set_relay_state(&mut self, relay_state: &RelayState);
    fn play_frequency(&mut self, frequency: &Frequency);
    fn wait(&mut self, delay_ms: f32);

    fn play_note(&mut self, note: &Note) {
        self.play_frequency(&note.freq);
        let total_delay = note.length.as_ms(BPM);
        let break_after_note = delay_after_note_ms(BPM);
        self.wait(total_delay - break_after_note);
        self.play_frequency(&Frequency::Silence);
        self.wait(break_after_note);
    }
}
