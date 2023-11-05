#![no_std]
pub mod hardware_interface;
pub mod melody;

extern crate alloc;

use alloc::vec;
use alloc::vec::Vec;

use hardware_interface::Frequency;
use hardware_interface::HardwareAction;
use hardware_interface::HardwareInterface;
use hardware_interface::Led;
use hardware_interface::LedState;
use hardware_interface::RelayState;
use hardware_interface::State;
use hardware_interface::Switch;
use melody::delay_after_note_ms;
use melody::Melody;
use melody::BPM;

use crate::melody::BEETHOVEN_9;

pub const NUM_MS_PER_SHOT: Time = 100;

pub type Time = u32;
pub type Duration = u32;

/// Higher level description of what should be done
enum Action {
    Pour(usize),
    FlashLed(Led, Duration),
    Play(&'static Melody),
}

struct TimedHardwareAction {
    timing_ms: Time,
    action: HardwareAction,
}

type Queue = Vec<TimedHardwareAction>;

fn get_relay_timing_ms(num: usize) -> u32 {
    (num as u32) * NUM_MS_PER_SHOT
}

struct Machine<H, P> {
    program: P,
    interface: H,
    actions: Queue,
    time: Time,
}

fn melody_to_hardware_actions(melody: &Melody) -> impl Iterator<Item = TimedHardwareAction> + '_ {
    let mut offset = 0;
    melody.iter().flat_map(move |note| {
        let total_delay = note.length.as_ms(BPM);
        let break_after_note = delay_after_note_ms(BPM);
        let note_length = (total_delay - break_after_note) as Time;
        offset += total_delay as Time;
        [
            TimedHardwareAction {
                action: HardwareAction::PlayFrequency(note.freq.clone()),
                timing_ms: offset,
            },
            TimedHardwareAction {
                action: HardwareAction::PlayFrequency(Frequency::Silence),
                timing_ms: offset + note_length,
            },
        ]
        .into_iter()
    })
}

impl<H: HardwareInterface, P: Program> Machine<H, P> {
    fn new(program: P, interface: H) -> Self {
        Self {
            program,
            interface,
            actions: Queue::default(),
            time: 0,
        }
    }

    fn add(&mut self, action: Action) {
        let mut make_action_in_ms_from_now = |ms: Duration, action: HardwareAction| {
            self.actions.push(TimedHardwareAction {
                timing_ms: self.time + ms,
                action,
            })
        };
        match action {
            Action::Pour(num) => {
                make_action_in_ms_from_now(0, HardwareAction::SetRelayState(RelayState::On));
                make_action_in_ms_from_now(
                    get_relay_timing_ms(num),
                    HardwareAction::SetRelayState(RelayState::On),
                );
            }
            Action::FlashLed(led, duration) => {
                make_action_in_ms_from_now(0, HardwareAction::SetLedState(led, LedState::On));
                make_action_in_ms_from_now(
                    duration,
                    HardwareAction::SetLedState(led, LedState::On),
                );
            }
            Action::Play(melody) => {
                for action in melody_to_hardware_actions(melody) {
                    make_action_in_ms_from_now(action.timing_ms, action.action);
                }
            }
        }
    }

    fn perform_pending_actions(&mut self) {
        let (actions_to_perform, remaining_actions): (Queue, Queue) = self
            .actions
            .drain(..)
            .partition(|action| action.timing_ms <= self.time);
        self.actions = remaining_actions;
        for action in actions_to_perform {
            self.interface.perform_action(action.action);
        }
    }

    fn run(mut self) -> ! {
        let mut state = State::new();
        loop {
            self.time = self.interface.get_elapsed_time_ms();
            state = self.interface.update_state(state);
            for action in self.program.get_new_actions(&state) {
                self.add(action);
            }
            self.perform_pending_actions();
        }
    }
}

trait Program {
    fn get_new_actions(&mut self, state: &State) -> Vec<Action>;
}

struct SimplePouring;

impl Program for SimplePouring {
    fn get_new_actions(&mut self, state: &State) -> Vec<Action> {
        for num in 0..10 {
            if state.just_pressed(Switch::number(num)) {
                return vec![Action::Play(&BEETHOVEN_9)];
            }
        }
        vec![]
    }
}

pub fn main_loop(interface: impl HardwareInterface) -> ! {
    Machine::new(SimplePouring, interface).run()
}
