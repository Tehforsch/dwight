use alloc::boxed::Box;

use crate::hardware_interface::Frequency;
use crate::hardware_interface::Led;
use crate::hardware_interface::RelayState;
use crate::hardware_interface::State;
use crate::hardware_interface::Switch;
use crate::melody::Melody;
use crate::melody::BEETHOVEN_5;
use crate::melody::BEETHOVEN_9;
use crate::melody::CHROMATIC_SCALE;
use crate::melody::CONFIRM_SELECTION;
use crate::melody::PROGRAM_SWITCHING;
use crate::melody::RUSSIAN_ROULETTE_PLAYER_SELECTED;
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
                machine.play_melody(&CHROMATIC_SCALE[..num]);
                machine.pour(num);
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
}

#[derive(Default)]
struct RussianRoulette {
    num_players: Option<usize>,
    state: RussianRouletteGameState,
}

#[derive(Default)]
enum RussianRouletteGameState {
    #[default]
    PlayerSelection,
    AwaitingGlass,
}

impl RussianRoulette {
    fn randomly_select_player(&self) -> bool {
        true
        // let probability = 1.0 / self.num_players.unwrap() as f32;
    }
}

impl Program for RussianRoulette {
    fn update(&mut self, machine: &mut Machine, state: &State) {
        if self.num_players.is_none() {
            if let Some(num) = state.lowest_pressed_number_key() {
                if num == 0 {
                } else {
                    machine.play_melody(CONFIRM_SELECTION);
                    machine.wait_for_all_actions();
                    self.num_players = Some(num);
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
        if state.anything_pressed() {
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
            let num_shots = 5; // todo randomize
            machine.pour(num_shots);
            self.state = RussianRouletteGameState::PlayerSelection;
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
        Switch::Number3 => Some((BEETHOVEN_5, Box::new(RussianRoulette::default()))),
        _ => None,
    }
}
