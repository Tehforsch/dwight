use crate::hardware_interface::Frequency;
use crate::Duration;

pub const BREAK_AFTER_EACH_NOTE_IN_QUARTER_NOTES: f32 = 0.25;

pub enum Length {
    Half,
    Quarter,
    Eighth,
    Sixteenth,
}

impl Length {
    const fn as_ms(&self, bpm: f32) -> f32 {
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

    const fn from_num(num: usize) -> Self {
        match num {
            2 => Self::Half,
            4 => Self::Quarter,
            8 => Self::Eighth,
            16 => Self::Sixteenth,
            _ => unimplemented!(),
        }
    }
}

const fn factor_to_ms(factor: f32, bpm: f32) -> f32 {
    let bps = bpm / 60.0;
    1000.0 / bps * factor
}

const fn delay_after_note_ms(bpm: f32) -> f32 {
    factor_to_ms(BREAK_AFTER_EACH_NOTE_IN_QUARTER_NOTES, bpm)
}

pub struct Note {
    pub freq: Frequency,
    pub note_length: Duration,
    pub delay_after: Duration,
}

impl Note {
    pub fn total_length(&self) -> Duration {
        self.note_length + self.delay_after
    }
}

pub type Melody = [Note];

macro_rules! make_melody {
    ($name: ident, $bpm: literal, [$(( $note: ident, $length: literal)),* $(,)?]) => {
        pub const $name: &'static Melody =
            &[
                $(
                    Note {
                        freq: Frequency::$note,
                        note_length: Length::from_num($length).as_ms($bpm) as Duration,
                        delay_after: delay_after_note_ms($bpm) as Duration,
                    }
                ),*
            ];
    }
}

#[rustfmt::skip]
make_melody!(
    BEETHOVEN_9,
    200.0,
    [
        (E4, 8),
        (E4, 8),
        (F4, 8),
        (G4, 8),
        (G4, 8),
        (F4, 8),
        (E4, 8),
        (D4, 8),
        (C4, 8),
        (C4, 8),
        (D4, 8),
        (E4, 8),
        (E4, 4),
        (D4, 16),
        (D4, 4),
    ]
);

#[rustfmt::skip]
make_melody!(
    BEETHOVEN_5,
    140.0,
    [
        (E4, 16),
        (E4, 16),
        (E4, 16),
        (C4, 2),
        (D4, 16),
        (D4, 16),
        (D4, 16),
        (B4, 2),
    ]
);

#[rustfmt::skip]
make_melody!(
    PROGRAM_SWITCHING,
    300.0,
    [
        (C4, 16),
        (E4, 16),
        (G4, 16),
        (C4, 16),
        (E4, 16),
        (G4, 16),
        (C4, 16),
        (E4, 16),
        (G4, 16),
    ]
);
