use crate::hardware_interface::Frequency;
use crate::hardware_interface::RelayState;
use crate::hardware_interface::State;
use crate::melody::BEETHOVEN_9;
use crate::Machine;

pub trait Program {
    fn update(&mut self, machine: &mut Machine, state: &State);
}

pub struct SimplePouring;

impl Program for SimplePouring {
    fn update(&mut self, machine: &mut Machine, state: &State) {
        if state.iter_just_pressed().count() > 0 {
            machine.play_melody(&BEETHOVEN_9);
        }
    }
}

pub struct ContinuousPouring;

impl Program for ContinuousPouring {
    fn update(&mut self, machine: &mut Machine, state: &State) {
        if state.iter_just_pressed().count() > 0 {
            machine.set_relay_state(RelayState::On);
            machine.set_speaker_frequency(Frequency::Some(400.0));
        } else {
            machine.set_relay_state(RelayState::Off);
        }
    }
}
