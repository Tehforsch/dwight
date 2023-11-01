const fn calc_note(freq: f32) -> u16 {
    (12_000_000 as f32 / 40 as f32 / freq) as u16
}

pub const C4: u16 = calc_note(261.63);
pub const D4: u16 = calc_note(293.66);
pub const E4: u16 = calc_note(329.63);
pub const F4: u16 = calc_note(349.23);
pub const G4: u16 = calc_note(392.00);
pub const A4: u16 = calc_note(440.00);
pub const NO_NOTE: u16 = calc_note(0.0);
