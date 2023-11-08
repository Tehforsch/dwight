use alloc::vec::Vec;

use hardware_interface::Action;
use hardware_interface::Frequency;
use hardware_interface::HardwareInterface;
use hardware_interface::Led;
use hardware_interface::LedState;
use hardware_interface::RelayState;
use hardware_interface::State;
use melody::Melody;
use melody::CHROMATIC_SCALE;
use programs::Program;

use crate::hardware_interface;
use crate::melody;
use crate::programs;
use crate::Duration;
use crate::Time;

pub const NUM_MS_PER_SHOT: Time = 100;

#[derive(Debug)]
struct TimedAction {
    timing_ms: Time,
    action: Action,
}

type Queue = Vec<TimedAction>;

fn get_relay_timing_ms(num: usize) -> u32 {
    (num as u32) * NUM_MS_PER_SHOT
}

pub struct Machine {
    actions: Queue,
    time_ms: Time,
    wait_for_all_actions: bool,
}

impl Machine {
    pub fn new() -> Self {
        Self {
            actions: Queue::default(),
            time_ms: 0,
            wait_for_all_actions: false,
        }
    }

    pub fn time_ms(&self) -> Time {
        self.time_ms
    }

    fn queue_action(&mut self, ms: Duration, action: Action) {
        self.actions.push(TimedAction {
            timing_ms: self.time_ms + ms,
            action,
        })
    }

    fn perform_pending_actions(&mut self, interface: &mut impl HardwareInterface) {
        let (actions_to_perform, remaining_actions): (Queue, Queue) = self
            .actions
            .drain(..)
            .partition(|action| action.timing_ms <= self.time_ms);
        self.actions = remaining_actions;
        for action in actions_to_perform {
            interface.perform_action(action.action);
        }
    }

    pub fn run(mut self, mut interface: impl HardwareInterface, mut program: impl Program) -> ! {
        let mut state = State::new();
        loop {
            self.time_ms = interface.get_elapsed_time_ms();
            state = interface.update_state(state);
            if self.wait_for_all_actions {
                self.wait_for_all_actions = !self.actions.is_empty();
            } else {
                program.update(&mut self, &state);
            }
            self.perform_pending_actions(&mut interface);
        }
    }

    pub fn pour(&mut self, num: usize) {
        self.queue_action(0, Action::SetRelayState(RelayState::On));
        self.queue_action(
            get_relay_timing_ms(num),
            Action::SetRelayState(RelayState::Off),
        );
    }

    pub fn pour_with_melody(&mut self, num: usize) {
        self.pour(num);
        let num_notes_to_play = num.min(CHROMATIC_SCALE.len());
        self.play_melody(&CHROMATIC_SCALE[..num_notes_to_play]);
    }

    pub fn flash_led(&mut self, led: Led, duration: Duration) {
        self.queue_action(0, Action::SetLedState(led, LedState::on()));
        self.queue_action(duration, Action::SetLedState(led, LedState::off()));
    }

    pub fn play_melody(&mut self, melody: &Melody) {
        let mut offset = 0;
        for note in melody.iter() {
            self.queue_action(offset, Action::SetSpeakerFrequency(note.freq.clone()));
            self.queue_action(
                offset + note.note_length,
                Action::SetSpeakerFrequency(Frequency::Silence),
            );
            offset += note.total_length();
        }
    }

    pub fn wait_for_all_actions(&mut self) {
        self.wait_for_all_actions = true;
    }

    pub fn set_relay_state(&mut self, state: RelayState) {
        self.queue_action(0, Action::SetRelayState(state));
    }

    pub fn set_speaker_frequency(&mut self, freq: Frequency) {
        self.queue_action(0, Action::SetSpeakerFrequency(freq));
    }

    pub fn set_led_state(&mut self, led: Led, state: LedState) {
        self.queue_action(0, Action::SetLedState(led, state));
    }
}
