#![no_std]

pub mod machine;
pub mod melody;

use machine::{Led, LedState, Machine, Switch};
use melody::beethoven_9;

pub fn main_loop(mut machine: impl Machine) -> ! {
    let melody = beethoven_9();
    loop {
        if machine.is_pressed(Switch::Number(0)) {
            machine.set_led_state(Led::Left, LedState::On);
        } else if machine.is_pressed(Switch::Number(1)) {
            machine.set_led_state(Led::Left, LedState::Off);
        } else if machine.is_pressed(Switch::Left) {
            for note in melody.iter() {
                machine.play_note(&note);
            }
        }
    }
}
