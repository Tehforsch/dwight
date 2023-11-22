use alloc::vec;
use alloc::vec::Vec;
use core::ops::RangeInclusive;

use crate::hardware_interface::State;
use crate::hardware_interface::Switch;
use crate::machine::Machine;
use crate::melody::CONFIRM_SELECTION;
use crate::melody::ERROR;
use crate::programs::Program;
use crate::Duration;
use crate::Time;

const DEFAULT_NUM_PLAYERS: usize = 2;

const DURATION_MS_PER_SHOT: Time = 700;

const RUSSIAN_ROULETTE_DEFAULT_LOSS_PROBABILITY: f32 = 0.1;
const RUSSIAN_ROULETTE_DEFAULT_MIN_NUM_SHOTS: usize = 4;
const RUSSIAN_ROULETTE_DEFAULT_MAX_NUM_SHOTS: usize = 10;

const REACTION_DEFAULT_NUM_SHOTS_LOSER: usize = 5;
const REACTION_DEFAULT_NUM_SHOTS_EARLY_START: usize = 10;

#[derive(Debug)]
pub struct Configuration {
    pub num_players: usize,
    pub shot_duration: Duration,
    pub russian_roulette_loss_probability: f32,
    pub russian_roulette_min_num_shots: usize,
    pub russian_roulette_max_num_shots: usize,
    pub reaction_num_shots_loser: usize,
    pub reaction_num_shots_early_start: usize,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            num_players: DEFAULT_NUM_PLAYERS,
            shot_duration: DURATION_MS_PER_SHOT,
            russian_roulette_loss_probability: RUSSIAN_ROULETTE_DEFAULT_LOSS_PROBABILITY,
            russian_roulette_min_num_shots: RUSSIAN_ROULETTE_DEFAULT_MIN_NUM_SHOTS,
            russian_roulette_max_num_shots: RUSSIAN_ROULETTE_DEFAULT_MAX_NUM_SHOTS,
            reaction_num_shots_loser: REACTION_DEFAULT_NUM_SHOTS_LOSER,
            reaction_num_shots_early_start: REACTION_DEFAULT_NUM_SHOTS_EARLY_START,
        }
    }
}

enum Variable {
    ReactionNumberOfPlayers,
    DelayPerShot,
    RussianRouletteLossProbability,
    RussianRouletteMinNumberOfShots,
    RussianRouletteMaxNumberOfShots,
    ReactionNumShotsLoser,
    ReactionNumShotsEarlyStart,
}

impl Variable {
    fn acceptable_range(&self) -> RangeInclusive<usize> {
        match self {
            Variable::ReactionNumberOfPlayers => 1..=9,
            Variable::DelayPerShot => 100..=2000,
            Variable::RussianRouletteLossProbability => 0..=100,
            Variable::RussianRouletteMinNumberOfShots => 1..=80,
            Variable::RussianRouletteMaxNumberOfShots => 1..=80,
            Variable::ReactionNumShotsLoser => 1..=80,
            Variable::ReactionNumShotsEarlyStart => 1..=80,
        }
    }
}

#[derive(Default)]
pub struct ConfigurationProgram {
    selected_variable: Option<Variable>,
    typed_digits: Vec<usize>,
}

impl ConfigurationProgram {
    fn configure(&self, machine: &mut Machine, selected_variable: &Variable, num: usize) {
        let config = machine.get_config_mut();
        match selected_variable {
            Variable::ReactionNumberOfPlayers => {
                config.num_players = num;
            }
            Variable::DelayPerShot => {
                config.shot_duration = num as u32;
            }
            Variable::RussianRouletteLossProbability => {
                config.russian_roulette_loss_probability = num as f32 / 100.0;
            }
            Variable::RussianRouletteMinNumberOfShots => {
                config.russian_roulette_min_num_shots = num;
            }
            Variable::RussianRouletteMaxNumberOfShots => {
                config.russian_roulette_max_num_shots = num;
            }
            Variable::ReactionNumShotsLoser => {
                config.reaction_num_shots_loser = num;
            }
            Variable::ReactionNumShotsEarlyStart => {
                config.reaction_num_shots_early_start = num;
            }
        }
    }

    fn wait_for_setting(&mut self, machine: &mut Machine, state: &State) {
        let selected_variable = self.selected_variable.as_ref().unwrap();
        for switch in state.iter_just_pressed() {
            if let Some(num) = switch.get_num() {
                self.typed_digits.push(num);
            }
        }
        if state.just_pressed(Switch::Right) {
            let num = self.get_typed_num();
            if selected_variable.acceptable_range().contains(&num) {
                machine.play_melody(CONFIRM_SELECTION);
                self.configure(machine, &selected_variable, num);
                self.reset();
            } else {
                machine.play_melody(ERROR);
                self.typed_digits = vec![];
            }
            machine.wait_for_all_actions();
        }
        if state.just_pressed(Switch::Left) {
            self.reset();
        }
    }

    fn reset(&mut self) {
        self.typed_digits = vec![];
        self.selected_variable = None;
    }

    fn get_typed_num(&self) -> usize {
        let mut factor = 1;
        let mut sum = 0;
        for digit in self.typed_digits.iter().rev() {
            sum += digit * factor;
            factor *= 10;
        }
        sum
    }

    fn wait_for_variable_selection(&mut self, machine: &mut Machine, state: &State) {
        if let Some(num) = state.lowest_pressed_number_key() {
            self.selected_variable = match num {
                1 => Some(Variable::DelayPerShot),
                2 => Some(Variable::RussianRouletteLossProbability),
                3 => Some(Variable::RussianRouletteMinNumberOfShots),
                4 => Some(Variable::RussianRouletteMaxNumberOfShots),
                5 => Some(Variable::ReactionNumberOfPlayers),
                6 => Some(Variable::ReactionNumShotsLoser),
                7 => Some(Variable::ReactionNumShotsEarlyStart),
                _ => None,
            };
            if self.selected_variable.is_some() {
                machine.play_melody(CONFIRM_SELECTION);
                machine.wait_for_all_actions();
            }
        }
    }
}

impl Program for ConfigurationProgram {
    fn update(&mut self, machine: &mut Machine, state: &State) {
        if self.selected_variable.is_some() {
            self.wait_for_setting(machine, state);
        } else {
            self.wait_for_variable_selection(machine, state);
        }
    }
}
