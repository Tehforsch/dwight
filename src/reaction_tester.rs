use alloc::vec::Vec;

use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;

use crate::hardware_interface::Led;
use crate::hardware_interface::State;
use crate::hardware_interface::Switch;
use crate::machine::Machine;
use crate::melody::REACTION_TESTER_EARLY_START_MELODY;
use crate::melody::REACTION_TESTER_GAME_BEGINS_MELODY;
use crate::melody::REACTION_TESTER_PLAYER_0_MELODY_IDENTIFICATION_MELODY;
use crate::melody::REACTION_TESTER_PLAYER_1_MELODY_IDENTIFICATION_MELODY;
use crate::melody::REACTION_TESTER_PLAYER_2_MELODY_IDENTIFICATION_MELODY;
use crate::melody::REACTION_TESTER_TEAM_WON_MELODY;
use crate::melody::REACTION_TESTER_WAIT_FOR_REACTION_MELODY;
use crate::programs::Program;
use crate::Duration;
use crate::Time;

const MIN_REACTION_DURATION_MS: u32 = 5000;
const MAX_REACTION_DURATION_MS: u32 = 15000;

const LED_ON_DURATION: Duration = 200;
const LED_FLASH_DURATION: Duration = 500;

const MAX_NUM_PLAYERS: usize = 6;

#[derive(Debug)]
struct TeamState {
    players_pressed: Vec<bool>,
}

impl TeamState {
    fn new(num_players: usize) -> TeamState {
        TeamState {
            players_pressed: (0..num_players).map(|_| false).collect(),
        }
    }

    fn won(&self) -> bool {
        self.players_pressed.iter().all(|x| *x)
    }
}

enum Reason {
    SlowReaction,
    EarlyStart(usize),
}

enum GameState {
    WaitForStart,
    WaitForTiming(Time),
    WaitForAllButtonPresses(TeamState, TeamState),
    WaitForGlass { reason: Reason, team: Team },
}

#[derive(Debug)]
enum Team {
    Left,
    Right,
}

#[derive(Debug)]
struct Player {
    button_num: usize,
    index: usize,
}

const PLAYERS_LEFT_SIDE: &[Player] = &[
    Player {
        button_num: 1,
        index: 0,
    },
    Player {
        button_num: 4,
        index: 1,
    },
    Player {
        button_num: 7,
        index: 2,
    },
];

const PLAYERS_RIGHT_SIDE: &[Player] = &[
    Player {
        button_num: 3,
        index: 0,
    },
    Player {
        button_num: 6,
        index: 1,
    },
    Player {
        button_num: 9,
        index: 2,
    },
];

pub struct ReactionTester {
    num_players: usize,
    state: GameState,
    rng: SmallRng,
}

fn get_wait_for_start_state_with_random_timing(machine: &Machine, rng: &mut SmallRng) -> GameState {
    let time = machine.time_ms();
    let duration = rng.gen_range(MIN_REACTION_DURATION_MS..MAX_REACTION_DURATION_MS);
    GameState::WaitForTiming(time + duration)
}

impl ReactionTester {
    pub fn new(machine: &Machine) -> Self {
        let num_players = machine.config().num_players.min(MAX_NUM_PLAYERS);
        Self {
            num_players,
            rng: SmallRng::seed_from_u64(machine.time_ms() as u64),
            state: GameState::WaitForStart,
        }
    }

    fn iter_active_players(&self) -> impl Iterator<Item = (&'static Player, Team)> {
        PLAYERS_LEFT_SIDE[..self.num_players_left()]
            .iter()
            .map(|player| (player, Team::Left))
            .chain(
                PLAYERS_RIGHT_SIDE[..self.num_players_right()]
                    .iter()
                    .map(|player| (player, Team::Right)),
            )
    }

    fn wait_for_start(
        &mut self,
        machine: &mut Machine,
        state: &State,
        timing: Time,
    ) -> Option<GameState> {
        let current_time = machine.time_ms();
        for (player, team) in self.iter_active_players() {
            if state.pressed(Switch::number(player.button_num)) {
                machine.play_melody(REACTION_TESTER_EARLY_START_MELODY);
                return Some(GameState::WaitForGlass {
                    team,
                    reason: Reason::EarlyStart(player.index),
                });
            }
        }
        if current_time > timing {
            machine.play_melody(REACTION_TESTER_WAIT_FOR_REACTION_MELODY);
            Some(GameState::WaitForAllButtonPresses(
                TeamState::new(self.num_players_left()),
                TeamState::new(self.num_players_right()),
            ))
        } else {
            None
        }
    }

    fn wait_for_glass(&mut self, machine: &mut Machine, state: &State) {
        let GameState::WaitForGlass { reason, team } = &self.state else {
            unreachable!()
        };
        let num_shots = match reason {
            Reason::SlowReaction => machine.config().reaction_num_shots_loser,
            Reason::EarlyStart(_) => machine.config().reaction_num_shots_early_start,
        };
        let led = match team {
            Team::Left => Led::Left,
            Team::Right => Led::Right,
        };
        if machine.no_ongoing_led_transition() {
            machine.flash_led(led, LED_FLASH_DURATION, LED_ON_DURATION);
        }
        if machine.no_sound_queued() {
            if let Reason::EarlyStart(player) = reason {
                match player {
                    0 => machine.play_melody(REACTION_TESTER_PLAYER_0_MELODY_IDENTIFICATION_MELODY),
                    1 => machine.play_melody(REACTION_TESTER_PLAYER_1_MELODY_IDENTIFICATION_MELODY),
                    2 => machine.play_melody(REACTION_TESTER_PLAYER_2_MELODY_IDENTIFICATION_MELODY),
                    _ => unreachable!(),
                }
            }
        }
        if state.anything_just_pressed() {
            let num_shots = num_shots;
            machine.pour_with_melody(num_shots);
            machine.wait_for_all_actions();
            self.state = GameState::WaitForStart;
        }
    }

    fn num_players_left(&self) -> usize {
        let num_players_left = self.num_players / 2;
        num_players_left
    }

    fn num_players_right(&self) -> usize {
        let num_players_left = self.num_players_left();
        let num_players_right = self.num_players - num_players_left;
        num_players_right
    }
}

fn wait_for_button_presses(
    machine: &mut Machine,
    state: &State,
    left: &mut TeamState,
    right: &mut TeamState,
) -> Option<GameState> {
    let left_won = update_and_get_victory_state(state, PLAYERS_LEFT_SIDE, left);
    let right_won = update_and_get_victory_state(state, PLAYERS_RIGHT_SIDE, right);
    if left_won {
        machine.play_melody(REACTION_TESTER_TEAM_WON_MELODY);
        Some(GameState::WaitForGlass {
            team: Team::Right,
            reason: Reason::SlowReaction,
        })
    } else if right_won {
        machine.play_melody(REACTION_TESTER_TEAM_WON_MELODY);
        Some(GameState::WaitForGlass {
            team: Team::Left,
            reason: Reason::SlowReaction,
        })
    } else {
        None
    }
}

fn update_and_get_victory_state(
    state: &State,
    players: &[Player],
    team_state: &mut TeamState,
) -> bool {
    for (i, player) in players.iter().enumerate() {
        if state.pressed(Switch::number(player.button_num)) {
            // Make sure we dont panic if somebody presses an out of bounds button
            if let Some(player_state) = team_state.players_pressed.get_mut(i) {
                *player_state = true;
            }
        }
    }
    team_state.won()
}

impl Program for ReactionTester {
    fn update(&mut self, machine: &mut Machine, state: &State) {
        match self.state {
            GameState::WaitForStart => {
                if self
                    .iter_active_players()
                    .all(|(player, _)| state.pressed(Switch::number(player.button_num)))
                {
                    self.state =
                        get_wait_for_start_state_with_random_timing(machine, &mut self.rng);
                    machine.play_melody(REACTION_TESTER_GAME_BEGINS_MELODY);
                    machine.wait_for_all_actions();
                }
            }
            GameState::WaitForTiming(time) => {
                if let Some(new_state) = self.wait_for_start(machine, state, time) {
                    self.state = new_state;
                    machine.wait_for_all_actions();
                }
            }
            GameState::WaitForAllButtonPresses(ref mut left, ref mut right) => {
                if let Some(new_state) = wait_for_button_presses(machine, state, left, right) {
                    self.state = new_state;
                    machine.wait_for_all_actions();
                }
            }
            GameState::WaitForGlass { .. } => self.wait_for_glass(machine, state),
        }
    }
}
