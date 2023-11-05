use smallvec::smallvec;
use smallvec::SmallVec;

use crate::machine::Frequency;

pub const BPM: f32 = 200.0;
pub const BREAK_AFTER_EACH_NOTE_IN_QUARTER_NOTES: f32 = 0.25;

pub const NUM_NOTES_NO_ALLOC: usize = 32;

pub enum Length {
    Half,
    Quarter,
    Eighth,
    Sixteenth,
}

fn factor_to_ms(factor: f32, bpm: f32) -> f32 {
    let bps = bpm / 60.0;
    1000.0 / bps * factor
}

impl Length {
    pub fn as_ms(&self, bpm: f32) -> f32 {
        factor_to_ms(
            match self {
                Length::Half => 2.0,
                Length::Quarter => 1.0,
                Length::Eighth => 0.5,
                Length::Sixteenth => 0.25,
            },
            bpm,
        )
    }

    pub fn from_num(num: usize) -> Self {
        match num {
            2 => Self::Half,
            4 => Self::Quarter,
            8 => Self::Eighth,
            16 => Self::Sixteenth,
            _ => unimplemented!(),
        }
    }
}

pub fn delay_after_note_ms(bpm: f32) -> f32 {
    factor_to_ms(BREAK_AFTER_EACH_NOTE_IN_QUARTER_NOTES, bpm)
}

pub struct Note {
    pub freq: Frequency,
    pub length: Length,
}

pub struct Melody {
    notes: SmallVec<[Note; NUM_NOTES_NO_ALLOC]>,
}

impl Melody {
    pub fn iter(&self) -> impl Iterator<Item = &Note> {
        self.notes.iter()
    }
}

macro_rules! make_melody {
    ($name: ident, [$(( $note: ident, $length: literal)),* $(,)?]) => {
        pub fn $name() -> Melody {
            Melody {
                notes: smallvec![
                    $(
                        Note {
                            freq: Frequency::$note,
                            length: Length::from_num($length),
                        }
                    ),*
                ]
            }
        }
    }
}

#[rustfmt::skip]
make_melody!(
    beethoven_9,
    [
        (E4, 4),
        (E4, 4),
        (F4, 4),
        (G4, 4),
        (G4, 4),
        (F4, 4),
        (E4, 4),
        (D4, 4),
        (C4, 4),
        (C4, 4),
        (D4, 4),
        (E4, 4),
        (E4, 2),
        (C4, 8),
        (C4, 2),
    ]
);
