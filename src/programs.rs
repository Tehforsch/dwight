use alloc::boxed::Box;

use rand::prelude::*;
use rand::rngs::SmallRng;
use rand::SeedableRng;

use crate::hardware_interface::Frequency;
use crate::hardware_interface::Led;
use crate::hardware_interface::RelayState;
use crate::hardware_interface::State;
use crate::hardware_interface::Switch;
use crate::melody::Melody;
use crate::melody::BEETHOVEN_5;
use crate::melody::BEETHOVEN_9;
use crate::melody::CONFIRM_SELECTION;
use crate::melody::ERROR;
use crate::melody::IN_PARIS;
use crate::melody::PROGRAM_SWITCHING;
use crate::melody::RUSSIAN_ROULETTE_PLAYER_SELECTED;
use crate::Duration;
use crate::Machine;

const PROGRAM_SWITCH_LED_ON_DURATION_MS: Duration = 500;
const PROGRAM_SWITCH_LED_TRANSITION_DURATION_MS: Duration = 500;

const MAX_SHOTS_RUSSIAN_ROULETTE: usize = 10;
const MIN_SHOTS_RUSSIAN_ROULETTE: usize = 1;
const BASE_PROBABILITY_RUSSIAN_ROULETTE: f64 = 0.5;

pub trait Program {
    fn update(&mut self, machine: &mut Machine, state: &State);
    fn cleanup_before_switch(&mut self, _machine: &mut Machine) {}
}

pub struct SimplePouring;

impl Program for SimplePouring {
    fn update(&mut self, machine: &mut Machine, state: &State) {
        for switch in state.iter_just_pressed() {
            if let Some(num) = switch.get_num() {
                machine.pour_with_melody(num);
                machine.wait_for_all_actions();
                return;
            }
        }
    }
}

pub struct ContinuousPouring;

impl Program for ContinuousPouring {
    fn update(&mut self, machine: &mut Machine, state: &State) {
        if state.anything_pressed() {
            machine.set_relay_state(RelayState::On);
            machine.set_speaker_frequency(Frequency::Some(400.0));
        } else {
            machine.set_relay_state(RelayState::Off);
            machine.set_speaker_frequency(Frequency::Silence);
        }
    }

    fn cleanup_before_switch(&mut self, machine: &mut Machine) {
        machine.set_relay_state(RelayState::Off);
        machine.set_speaker_frequency(Frequency::Silence);
    }
}

#[derive(Default)]
enum RussianRouletteGameState {
    #[default]
    PlayerSelection,
    AwaitingGlass,
}

#[derive(Default)]
struct RussianRoulette {
    num_players: Option<usize>,
    state: RussianRouletteGameState,
    rng: Option<SmallRng>,
}

impl RussianRoulette {
    fn randomly_select_player(&mut self) -> bool {
        let probability = BASE_PROBABILITY_RUSSIAN_ROULETTE / self.num_players.unwrap() as f64;
        self.rng.as_mut().unwrap().gen_bool(probability)
    }

    fn get_random_num_shots(&mut self) -> usize {
        self.rng
            .as_mut()
            .unwrap()
            .gen_range(MIN_SHOTS_RUSSIAN_ROULETTE..MAX_SHOTS_RUSSIAN_ROULETTE)
    }
}

impl Program for RussianRoulette {
    fn update(&mut self, machine: &mut Machine, state: &State) {
        if self.num_players.is_none() {
            if let Some(num) = state.lowest_pressed_number_key() {
                if num == 0 {
                    machine.play_melody(ERROR);
                } else {
                    machine.play_melody(CONFIRM_SELECTION);
                    machine.wait_for_all_actions();
                    self.num_players = Some(num);
                    self.rng = Some(SmallRng::seed_from_u64(machine.time_ms() as u64));
                }
            }
        } else {
            match self.state {
                RussianRouletteGameState::PlayerSelection => {
                    self.test_player_selection(machine, state)
                }
                RussianRouletteGameState::AwaitingGlass => self.wait_for_glass(machine, state),
            }
        }
    }
}

impl RussianRoulette {
    fn test_player_selection(&mut self, machine: &mut Machine, state: &State) {
        if state.anything_just_pressed() {
            let selected = self.randomly_select_player();
            if selected {
                machine.play_melody(RUSSIAN_ROULETTE_PLAYER_SELECTED);
                machine.wait_for_all_actions();
                self.state = RussianRouletteGameState::AwaitingGlass;
            }
        }
    }

    fn wait_for_glass(&mut self, machine: &mut Machine, state: &State) {
        if state.anything_just_pressed() {
            let num_shots = self.get_random_num_shots(); // todo randomize
            machine.pour_with_melody(num_shots);
            machine.wait_for_all_actions();
            self.state = RussianRouletteGameState::PlayerSelection;
        }
    }
}

struct LedTest;

impl Program for LedTest {
    fn update(&mut self, machine: &mut Machine, _: &State) {
        if machine.no_ongoing_led_transition() {
            machine.flash_led(Led::Left, 2000, 5000);
            machine.flash_led(Led::Right, 2000, 5000);
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
            program: Box::new(LedTest),
        }
    }
}

impl Program for ProgramSwitching {
    fn update(&mut self, machine: &mut Machine, state: &State) {
        if self.in_selection_mode {
            if machine.no_ongoing_led_transition() {
                machine.flash_led(
                    Led::Left,
                    PROGRAM_SWITCH_LED_TRANSITION_DURATION_MS,
                    PROGRAM_SWITCH_LED_ON_DURATION_MS,
                );
                machine.flash_led(
                    Led::Right,
                    PROGRAM_SWITCH_LED_TRANSITION_DURATION_MS,
                    PROGRAM_SWITCH_LED_ON_DURATION_MS,
                );
            }
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
                self.program.cleanup_before_switch(machine);
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
        Switch::Number3 => Some((IN_PARIS, Box::new(RussianRoulette::default()))),
        Switch::Number4 => Some((IN_PARIS, Box::new(LedTest))),
        _ => None,
    }
}
