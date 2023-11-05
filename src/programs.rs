use alloc::boxed::Box;

use crate::hardware_interface::Frequency;
use crate::hardware_interface::Led;
use crate::hardware_interface::RelayState;
use crate::hardware_interface::State;
use crate::hardware_interface::Switch;
use crate::melody::Melody;
use crate::melody::BEETHOVEN_5;
use crate::melody::BEETHOVEN_9;
use crate::melody::PROGRAM_SWITCHING;
use crate::Duration;
use crate::Machine;

pub trait Program {
    fn update(&mut self, machine: &mut Machine, state: &State);
}

pub struct SimplePouring;

impl Program for SimplePouring {
    fn update(&mut self, machine: &mut Machine, state: &State) {
        for switch in state.iter_just_pressed() {
            if let Some(num) = switch.get_num() {
                machine.play_melody(&BEETHOVEN_9);
                machine.pour(num);
            }
        }
    }
}

pub struct ContinuousPouring;

impl Program for ContinuousPouring {
    fn update(&mut self, machine: &mut Machine, state: &State) {
        if state.iter_pressed().count() > 0 {
            machine.set_relay_state(RelayState::On);
            machine.set_speaker_frequency(Frequency::Some(400.0));
        } else {
            machine.set_relay_state(RelayState::Off);
            machine.set_speaker_frequency(Frequency::Silence);
        }
    }
}

pub struct ProgramSwitching {
    in_selection_mode: bool,
    program: Box<dyn Program>,
}

impl Default for ProgramSwitching {
    fn default() -> Self {
        Self {
            in_selection_mode: false,
            program: Box::new(SimplePouring),
        }
    }
}

impl Program for ProgramSwitching {
    fn update(&mut self, machine: &mut Machine, state: &State) {
        const LED_FLASH_DURATION_MS: Duration = 500;

        if self.in_selection_mode {
            for switch in state.iter_just_pressed() {
                if let Some((melody, program)) = program_num(switch) {
                    self.program = program;
                    self.in_selection_mode = false;
                    machine.play_melody(melody);
                    machine.wait_for_all_actions();
                }
            }
        } else {
            if state.pressed(Switch::Left) && state.pressed(Switch::Right) {
                machine.flash_led(Led::Left, LED_FLASH_DURATION_MS);
                machine.flash_led(Led::Right, LED_FLASH_DURATION_MS);
                machine.play_melody(PROGRAM_SWITCHING);
                self.in_selection_mode = true;
            } else {
                self.program.update(machine, state);
            }
        }
    }
}

fn program_num(switch: Switch) -> Option<(&'static Melody, Box<dyn Program>)> {
    match switch {
        Switch::Number1 => Some((BEETHOVEN_5, Box::new(ContinuousPouring))),
        Switch::Number2 => Some((BEETHOVEN_9, Box::new(SimplePouring))),
        _ => None,
    }
}
