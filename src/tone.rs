use cortex_m::delay::Delay;
use embedded_hal::PwmPin;
use rp_pico::hal::pwm::{FreeRunning, Pwm0, Slice};

/// External high-speed crystal on the Raspberry Pi Pico board is 12 MHz. Adjust
/// if your board has a different frequency
pub const XTAL_FREQ_HZ: u32 = 12_000_000u32;

const DIVIDER: i32 = 40;

pub const BPM: f32 = 120.0;

pub enum Length {
    Half,
    Quarter,
    Eighth,
}

impl Length {
    pub fn as_ms(&self, bpm: f32) -> f32 {
        let bps = bpm * 60.0;
        let factor = match self {
            Length::Half => 2.0,
            Length::Quarter => 1.0,
            Length::Eighth => 0.5,
        };
        1000.0 / bps * factor
    }
}

pub struct Note {
    freq: Option<f32>,
    length: Length,
}

impl Note {
    pub const C4: f32 = 261.63;
    pub const D4: f32 = 293.66;
    pub const E4: f32 = 329.63;
    pub const F4: f32 = 349.23;
    pub const G4: f32 = 392.00;
    pub const A4: f32 = 440.00;
    pub const B4: f32 = 493.88;

    pub fn playback(&self, pwm: &mut Slice<Pwm0, FreeRunning>, delay: &mut Delay) {
        if let Some(freq) = self.freq {
            let top = (XTAL_FREQ_HZ as f32 / DIVIDER as f32 / freq) as u16;
            pwm.channel_b.set_duty(top / 2);
            pwm.set_top(top);
        } else {
            pwm.channel_b.set_duty(0);
        };
        delay.delay_ms(self.length.as_ms(BPM) as u32);
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

pub fn my_favorite_melody() -> Melody<7> {
    Melody {
        notes: [
            Note {
                length: Length::Eighth,
                freq: Some(Note::C4),
            },
            Note {
                length: Length::Half,
                freq: Some(Note::D4),
            },
            Note {
                length: Length::Quarter,
                freq: Some(Note::E4),
            },
            Note {
                length: Length::Quarter,
                freq: Some(Note::F4),
            },
            Note {
                length: Length::Quarter,
                freq: Some(Note::G4),
            },
            Note {
                length: Length::Quarter,
                freq: Some(Note::A4),
            },
            Note {
                length: Length::Quarter,
                freq: Some(Note::B4),
            },
        ],
    }
}
