#![no_std]

pub mod machine;
pub mod melody;

use machine::Machine;
use melody::beethoven_9;

pub fn main_loop(mut machine: impl Machine) -> ! {
    let melody = beethoven_9();
    loop {
        for note in melody.iter() {
            machine.play_note(&note);
        }
    }
}
