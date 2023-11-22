use crate::hardware_interface::Frequency;
use crate::Duration;

pub const BREAK_AFTER_EACH_NOTE_IN_QUARTER_NOTES: f32 = 0.25;

pub enum Length {
    Half,
    Quarter,
    DottedEighth,
    Eighth,
    DottedSixteenth,
    Sixteenth,
}

impl Length {
    const fn as_ms(&self, bpm: f32) -> f32 {
        factor_to_ms(
            match self {
                Length::Half => 2.0,
                Length::Quarter => 1.0,
                Length::DottedEighth => 0.75,
                Length::Eighth => 0.5,
                Length::DottedSixteenth => 0.375,
                Length::Sixteenth => 0.25,
            },
            bpm,
        )
    }

    const fn from_num(num: usize) -> Self {
        match num {
            2 => Self::Half,
            4 => Self::Quarter,
            6 => Self::DottedEighth,
            8 => Self::Eighth,
            12 => Self::DottedSixteenth,
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
    250.0,
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
    200.0,
    [
        (F4, 16),
        (F4, 16),
        (F4, 16),
        (C_SHARP_4, 2),
        (D_SHARP_4, 16),
        (D_SHARP_4, 16),
        (D_SHARP_4, 16),
        (C4, 2),
    ]
);

#[rustfmt::skip]
make_melody!(
    REACTION_TESTER_WAIT_FOR_REACTION_MELODY,
    140.0,
    [
        (F4, 4),
    ]
);

#[rustfmt::skip]
make_melody!(
    REACTION_TESTER_TEAM_WON_MELODY,
    200.0,
    [
        (C4, 8),
        (D4, 8),
        (E4, 8),
        (F4, 8),
        (G4, 4),
        (G4, 4),
    ]
);

#[rustfmt::skip]
make_melody!(
    REACTION_TESTER_EARLY_START_MELODY,
    300.0,
    [
        (C_SHARP_4, 8),
        (G4, 8),
        (C_SHARP_4, 8),
        (G4, 8),
        (C_SHARP_4, 8),
        (G4, 8),
        (C_SHARP_4, 8),
        (G4, 8),
        (C_SHARP_4, 8),
        (G4, 8),
    ]
);

#[rustfmt::skip]
make_melody!(
    REACTION_TESTER_GAME_BEGINS_MELODY,
    200.0,
    [
        (C4, 4),
        (C4, 4),
        (C4, 4),
        (C4, 4),
    ]
);

#[rustfmt::skip]
make_melody!(
    REACTION_TESTER_PLAYER_0_MELODY_IDENTIFICATION_MELODY,
    300.0,
    [
        (D4, 16),
        (BREAK, 2),
    ]
);

#[rustfmt::skip]
make_melody!(
    REACTION_TESTER_PLAYER_1_MELODY_IDENTIFICATION_MELODY,
    300.0,
    [
        (D4, 16),
        (D4, 16),
        (BREAK, 2),
    ]
);

#[rustfmt::skip]
make_melody!(
    REACTION_TESTER_PLAYER_2_MELODY_IDENTIFICATION_MELODY,
    300.0,
    [
        (D4, 16),
        (D4, 16),
        (D4, 16),
        (BREAK, 2),
    ]
);

#[rustfmt::skip]
make_melody!(
    IN_PARIS,
    180.0,
    [
        (F4, 16),
        (C4, 16),
        (BREAK, 8),
        (G_SHARP_4, 16),
        (C4, 16),
        (BREAK, 8),
        (G4, 16),
        (C4, 16),
        (BREAK, 8),
        (F4, 16),
        (C4, 16),
        (C5, 16),
    ]
);

#[rustfmt::skip]
make_melody!(
    BARBIE_GIRL,
    240.0,
    [
        (G4, 8),
        (E4, 8),
        (G4, 8),
        (C5, 8),
        (A4, 2),

        (F4, 8),
        (D4, 8),
        (F4, 8),
        (B4, 8),
        (G4, 4),
        (F4, 8),
        (E4, 8),
    ]
);

#[rustfmt::skip]
make_melody!(
    JINGLE,
    200.0,
    [
        (C4, 16),
        (C4, 16),
        (C4, 8),
        (BREAK, 8),
        (C4, 16),
        (C4, 16),
        (C4, 8),
        (BREAK, 8),
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

#[rustfmt::skip]
make_melody!(
    CONFIRM_SELECTION,
    200.0,
    [
        (C4, 16),
        (G4, 4),
        (C4, 16),
        (G4, 4),
    ]
);

#[rustfmt::skip]
make_melody!(
    ERROR,
    200.0,
    [
        (F_SHARP_4, 16),
        (C4, 16),
    ]
);

#[rustfmt::skip]
make_melody!(
    RUSSIAN_ROULETTE_PLAYER_SELECTED,
    200.0,
    [
        (C4, 8),
        (F_SHARP_4, 8),
        (C4, 8),
        (F_SHARP_4, 8),
        (C4, 8),
        (F_SHARP_4, 8),
    ]
);

#[rustfmt::skip]
make_melody!(
    RUSSIAN_ROULETTE_PLAYER_NOT_SELECTED,
    200.0,
    [
        (C4, 4),
    ]
);

#[rustfmt::skip]
make_melody!(
    CHROMATIC_SCALE,
    100.0,
    [
        (C4, 8),
        (C_SHARP_4, 8),
        (D4, 8),
        (D_SHARP_4, 8),
        (E4, 8),
        (F4, 8),
        (F_SHARP_4, 8),
        (G4, 8),
        (G_SHARP_4, 8),
        (A4, 8),
        (A_SHARP_4, 8),
        (B4, 8),
    ]
);
