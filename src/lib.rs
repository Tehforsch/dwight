#![no_std]

pub mod hardware_interface;
pub mod melody;

use hardware_interface::{
    Frequency, HardwareAction, HardwareInterface, Led, LedState, RelayState, State, Switch,
};
use melody::{Note, BPM};
use smallvec::SmallVec;

pub const NUM_ACTIONS_NO_ALLOC: usize = 32;

pub type Time = u32;
pub type Duration = u32;

/// Higher level description of what should be done
enum Action {
    Pour(usize),
    FlashLed(Led, Duration),
    PlayNote(Note),
}

struct TimedHardwareAction {
    timing_ms: Time,
    action: HardwareAction,
}

type Queue = SmallVec<[TimedHardwareAction; NUM_ACTIONS_NO_ALLOC]>;

fn get_relay_timing_ms(num: usize) -> u32 {
    (num as u32) * 100
}

struct Machine<H, P> {
    program: P,
    interface: H,
    actions: Queue,
    time: Time,
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
            Action::PlayNote(note) => {
                make_action_in_ms_from_now(0, HardwareAction::PlayFrequency(note.freq));
                make_action_in_ms_from_now(
                    note.length.as_ms(BPM) as Duration,
                    HardwareAction::PlayFrequency(Frequency::Silence),
                );
            }
        }
    }

    fn act(&mut self) {
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
            let action = self.program.update(&state);
            if let Some(action) = action {
                self.add(action);
            }
            self.act();
        }
    }
}

trait Program {
    fn update(&mut self, state: &State) -> Option<Action>;
}

struct SimplePouring;

impl Program for SimplePouring {
    fn update(&mut self, state: &State) -> Option<Action> {
        for num in 0..10 {
            if state.just_pressed(Switch::number(num)) {
                return Some(Action::Pour(num));
            }
        }
        None
    }
}

pub fn main_loop(interface: impl HardwareInterface) -> ! {
    Machine::new(SimplePouring, interface).run()
}
