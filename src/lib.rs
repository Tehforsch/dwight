#![no_std]
#![feature(const_fn_floating_point_arithmetic)]
pub mod hardware_interface;
mod melody;
mod programs;

extern crate alloc;

use alloc::vec::Vec;

use hardware_interface::Action;
use hardware_interface::Frequency;
use hardware_interface::HardwareInterface;
use hardware_interface::Led;
use hardware_interface::LedState;
use hardware_interface::RelayState;
use hardware_interface::State;
use melody::Melody;
use programs::Program;
use programs::SimplePouring;

pub const NUM_MS_PER_SHOT: Time = 100;

pub type Time = u32;
pub type Duration = u32;

#[derive(Debug)]
struct TimedAction {
    timing_ms: Time,
    action: Action,
}

type Queue = Vec<TimedAction>;

fn get_relay_timing_ms(num: usize) -> u32 {
    (num as u32) * NUM_MS_PER_SHOT
}

struct Machine {
    actions: Queue,
    time: Time,
}

impl Machine {
    fn new() -> Self {
        Self {
            actions: Queue::default(),
            time: 0,
        }
    }

    fn queue_action(&mut self, ms: Duration, action: Action) {
        self.actions.push(TimedAction {
            timing_ms: self.time + ms,
            action,
        })
    }

    fn perform_pending_actions(&mut self, interface: &mut impl HardwareInterface) {
        let (actions_to_perform, remaining_actions): (Queue, Queue) = self
            .actions
            .drain(..)
            .partition(|action| action.timing_ms <= self.time);
        self.actions = remaining_actions;
        for action in actions_to_perform {
            interface.perform_action(action.action);
        }
    }

    fn run(mut self, mut interface: impl HardwareInterface, mut program: impl Program) -> ! {
        let mut state = State::new();
        loop {
            self.time = interface.get_elapsed_time_ms();
            state = interface.update_state(state);
            program.update(&mut self, &state);
            self.perform_pending_actions(&mut interface);
        }
    }

    pub fn pour(&mut self, num: usize) {
        self.queue_action(0, Action::SetRelayState(RelayState::On));
        self.queue_action(
            get_relay_timing_ms(num),
            Action::SetRelayState(RelayState::On),
        );
    }

    pub fn flash_led(&mut self, led: Led, duration: Duration) {
        self.queue_action(0, Action::SetLedState(led, LedState::On));
        self.queue_action(duration, Action::SetLedState(led, LedState::On));
    }

    pub fn play_melody(&mut self, melody: &Melody) {
        let mut offset = 0;
        for note in melody.notes.iter() {
            self.queue_action(offset, Action::SetSpeakerFrequency(note.freq.clone()));
            self.queue_action(
                offset + note.note_length,
                Action::SetSpeakerFrequency(Frequency::Silence),
            );
            offset += note.total_length();
        }
    }

    pub fn set_relay_state(&mut self, state: RelayState) {
        self.queue_action(0, Action::SetRelayState(state));
    }

    pub fn set_speaker_frequency(&mut self, freq: Frequency) {
        self.queue_action(0, Action::SetSpeakerFrequency(freq));
    }
}

pub fn main_loop(interface: impl HardwareInterface) -> ! {
    Machine::new().run(interface, SimplePouring)
}
