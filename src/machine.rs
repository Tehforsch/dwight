use alloc::vec::Vec;

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
const DEFAULT_NUM_PLAYERS: usize = 2;

pub struct Configuration {
    pub num_players: usize,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            num_players: DEFAULT_NUM_PLAYERS,
        }
    }
}

#[derive(Debug)]
enum Action {
    SetLedTransition(Led, Transition),
    SetRelayState(RelayState),
    SetSpeakerFrequency(Frequency),
}

#[derive(Debug)]
struct TimedAction {
    timing_ms: Time,
    action: Action,
}

type Queue = Vec<TimedAction>;

#[derive(Default, Debug)]
struct Transition {
    start_val: f32,
    end_val: f32,
    duration: Time,
}

impl Transition {
    fn get_current_val(&self, time_elapsed: Time) -> LedState {
        let frac = if time_elapsed <= 0 {
            0.0
        } else if time_elapsed >= self.duration {
            1.0
        } else {
            time_elapsed as f32 / self.duration as f32
        };
        LedState {
            brightness: self.start_val + (self.end_val - self.start_val) * frac,
        }
    }

    pub fn on_within(duration: Duration) -> Self {
        Self {
            start_val: 0.0,
            end_val: 1.0,
            duration,
        }
    }

    pub fn off_within(duration: Duration) -> Self {
        Self {
            start_val: 1.0,
            end_val: 0.0,
            duration,
        }
    }
}

#[derive(Default, Debug)]
struct StartedTransition {
    start_time_ms: Time,
    transition: Transition,
}

impl StartedTransition {
    fn get_current_val(&self, time_ms: Time) -> LedState {
        self.transition
            .get_current_val(time_ms - self.start_time_ms)
    }

    fn end_time(&self) -> Time {
        self.transition.duration + self.start_time_ms
    }
}

fn get_relay_timing_ms(num: usize) -> u32 {
    (num as u32) * NUM_MS_PER_SHOT
}

pub struct Machine {
    actions: Queue,
    time_ms: Time,
    wait_for_all_actions: bool,
    left_led_transition: StartedTransition,
    right_led_transition: StartedTransition,
    config: Configuration,
}

impl Machine {
    pub fn new() -> Self {
        Self {
            actions: Queue::default(),
            time_ms: 0,
            wait_for_all_actions: false,
            left_led_transition: StartedTransition::default(),
            right_led_transition: StartedTransition::default(),
            config: Configuration::default(),
        }
    }

    pub fn config(&self) -> &Configuration {
        &self.config
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
            match action.action {
                Action::SetLedTransition(Led::Left, transition) => {
                    self.left_led_transition = StartedTransition {
                        transition,
                        start_time_ms: action.timing_ms,
                    }
                }
                Action::SetLedTransition(Led::Right, transition) => {
                    self.right_led_transition = StartedTransition {
                        transition,
                        start_time_ms: action.timing_ms,
                    }
                }
                Action::SetRelayState(state) => interface.set_relay_state(state),
                Action::SetSpeakerFrequency(freq) => interface.set_speaker_frequency(&freq),
            }
        }
    }

    fn update_leds(&mut self, interface: &mut impl HardwareInterface) {
        interface.set_led_state(
            Led::Left,
            self.left_led_transition.get_current_val(self.time_ms),
        );
        interface.set_led_state(
            Led::Right,
            self.right_led_transition.get_current_val(self.time_ms),
        );
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
            self.update_leds(&mut interface);
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

    pub fn flash_led(&mut self, led: Led, transition_duration: Duration, on_duration: Duration) {
        self.queue_action(
            0,
            Action::SetLedTransition(led, Transition::on_within(transition_duration)),
        );
        self.queue_action(
            transition_duration + on_duration,
            Action::SetLedTransition(led, Transition::off_within(transition_duration)),
        );
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

    pub fn no_ongoing_led_transition(&self) -> bool {
        self.time_ms > self.left_led_transition.end_time()
            && self.time_ms > self.right_led_transition.end_time()
    }

    pub fn configure_num_players(&mut self, num: usize) {
        self.config.num_players = num;
    }
}
