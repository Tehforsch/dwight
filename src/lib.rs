#![no_std]

pub mod hardware_interface;
pub mod melody;

use core::time::Duration;

use hardware_interface::{HardwareInterface, Led, State, Switch};
use melody::Note;
use smallvec::SmallVec;

pub const NUM_ACTIONS_NO_ALLOC: usize = 32;

enum Action {
    Pour(usize),
    FlashLed(Led, Duration),
    PlayNote(Note),
}

#[derive(Default)]
pub struct ActionQueue {
    actions: SmallVec<[Action; NUM_ACTIONS_NO_ALLOC]>,
}

impl ActionQueue {
    fn add(&mut self, action: Action) {
        self.actions.push(action);
    }

    fn act(&mut self, interface: &mut impl HardwareInterface) {}
}

pub struct Machine<H, P> {
    program: P,
    interface: H,
}

impl<H, P> Machine<H, P> {
    pub fn new(program: P, interface: H) -> Self {
        Self { program, interface }
    }
}

impl<H: HardwareInterface, P: Program> Machine<H, P> {
    pub fn run(mut self) -> ! {
        let mut queue = ActionQueue::default();
        let mut state = State::new();
        loop {
            state = self.interface.update_state(state);
            self.program.update(&mut queue, &state);
            queue.act(&mut self.interface);
        }
    }
}

pub trait Program {
    fn update(&mut self, queue: &mut ActionQueue, state: &State);
}

pub struct SimplePouring;

impl Program for SimplePouring {
    fn update(&mut self, queue: &mut ActionQueue, state: &State) {
        for num in 0..10 {
            if state.just_pressed(Switch::number(num)) {
                queue.add(Action::Pour(num));
            }
        }
    }
}
