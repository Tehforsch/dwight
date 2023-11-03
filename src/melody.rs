use cortex_m::delay::Delay;
use embedded_hal::PwmPin;
use rp_pico::hal::pwm::{FreeRunning, Pwm0, Slice};

/// External high-speed crystal on the Raspberry Pi Pico board is 12 MHz. Adjust
/// if your board has a different frequency
pub const XTAL_FREQ_HZ: u32 = 12_000_000u32;

const DIVIDER: i32 = 40;

pub const BPM: f32 = 200.0;
pub const BREAK_AFTER_EACH_NOTE_IN_QUARTER_NOTES: f32 = 0.25;

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

fn delay_after_note_ms(bpm: f32) -> f32 {
    factor_to_ms(BREAK_AFTER_EACH_NOTE_IN_QUARTER_NOTES, bpm)
}

pub struct Note {
    pub freq: Option<f32>,
    pub length: Length,
}

impl Note {
    pub const C4: Option<f32> = Some(261.63);
    pub const D4: Option<f32> = Some(293.66);
    pub const E4: Option<f32> = Some(329.63);
    pub const F4: Option<f32> = Some(349.23);
    pub const G4: Option<f32> = Some(392.00);
    pub const A4: Option<f32> = Some(440.00);
    pub const B4: Option<f32> = Some(493.88);
    pub const BREAK: Option<f32> = None;

    pub fn playback(&self, pwm: &mut Slice<Pwm0, FreeRunning>, delay: &mut Delay) {
        if let Some(freq) = self.freq {
            let top = (XTAL_FREQ_HZ as f32 / DIVIDER as f32 / freq) as u16;
            pwm.channel_b.set_duty(top / 2);
            pwm.set_top(top);
        } else {
            pwm.channel_b.set_duty(0);
        };
        let total_delay = self.length.as_ms(BPM);
        let break_after_note = delay_after_note_ms(BPM);
        delay.delay_ms((total_delay - break_after_note) as u32);
        pwm.channel_b.set_duty(0);
        delay.delay_ms(break_after_note as u32);
    }
}

pub struct Melody<const U: usize> {
    notes: [Note; U],
}

impl<const U: usize> Melody<U> {
    pub fn iter(&self) -> impl Iterator<Item = &Note> {
        self.notes.iter()
    }
}

macro_rules! count {
    () => (0usize);
    ( $x:tt $($xs:tt)* ) => (1usize + count!($($xs)*));
}

macro_rules! make_melody {
    ($name: ident, [$( $note: ident, $length: literal),* $(,)?]) => {
        pub fn $name() -> Melody<{ count!($($note)*) }> {
            Melody {
                notes: [
                    $(
                        Note {
                            freq: Note::$note,
                            length: Length::from_num($length),
                        }
                    ),*
                ],
            }
        }
    }
}

// make_melody!(
//     beethoven_9,
//     [
//         E4, 4, E4, 4, F4, 4, G4, 4, G4, 4, F4, 4, E4, 4, D4, 4, C4, 4, C4, 4, D4, 4, E4, 4, D4, 2,
//         C4, 8, C4, 2
//     ]
// );

make_melody!(beethoven_9, [E4, 4]);
