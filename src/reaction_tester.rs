use alloc::vec::Vec;

use rand::rngs::SmallRng;
use rand::Rng;
use rand::SeedableRng;

use crate::hardware_interface::Led;
use crate::hardware_interface::State;
use crate::hardware_interface::Switch;
use crate::machine::Machine;
use crate::melody::REACTION_TESTER_TEAM_WON_MELODY;
use crate::melody::REACTION_TESTER_WAIT_FOR_REACTION_MELODY;
use crate::programs::Program;
use crate::Duration;
use crate::Time;

const MIN_REACTION_DURATION_MS: u32 = 2000;
const MAX_REACTION_DURATION_MS: u32 = 20000;

const NUM_SHOTS_SLOW_REACTION: usize = 5;
const NUM_SHOTS_EARLY_START: usize = 10;

const LED_ON_DURATION: Duration = 200;
const LED_FLASH_DURATION: Duration = 500;

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
    EarlyStart,
}

enum GameState {
    WaitForStart(Time),
    WaitForAllButtonPresses(TeamState, TeamState),
    WaitForGlass { reason: Reason, team: Team },
}

enum Team {
    Left,
    Right,
}

struct ReactionTesterPlayer {
    button_num: usize,
}

const PLAYERS_LEFT_SIDE: &[ReactionTesterPlayer] = &[
    ReactionTesterPlayer { button_num: 1 },
    ReactionTesterPlayer { button_num: 4 },
    ReactionTesterPlayer { button_num: 7 },
];

const PLAYERS_RIGHT_SIDE: &[ReactionTesterPlayer] = &[
    ReactionTesterPlayer { button_num: 3 },
    ReactionTesterPlayer { button_num: 6 },
    ReactionTesterPlayer { button_num: 9 },
];

pub struct ReactionTester {
    num_players: usize,
    state: GameState,
    rng: SmallRng,
}

fn get_wait_for_start_state_with_random_timing(machine: &Machine, rng: &mut SmallRng) -> GameState {
    let time = machine.time_ms();
    let duration = rng.gen_range(MIN_REACTION_DURATION_MS..MAX_REACTION_DURATION_MS);
    GameState::WaitForStart(time + duration)
}

impl ReactionTester {
    pub fn new(machine: &Machine) -> Self {
        let mut rng = SmallRng::seed_from_u64(machine.time_ms() as u64);
        let state = get_wait_for_start_state_with_random_timing(machine, &mut rng);
        Self {
            num_players: machine.config().num_players,
            rng,
            state,
        }
    }

    fn wait_for_start(
        &mut self,
        machine: &mut Machine,
        state: &State,
        timing: Time,
    ) -> Option<GameState> {
        let current_time = machine.time_ms();
        let num_players_left = self.num_players / 2;
        let num_players_right = self.num_players - num_players_left;
        for (player, team) in PLAYERS_LEFT_SIDE
            .iter()
            .map(|player| (player, Team::Left))
            .chain(
                PLAYERS_RIGHT_SIDE
                    .iter()
                    .map(|player| (player, Team::Right)),
            )
        {
            if state.pressed(Switch::number(player.button_num)) {
                return Some(GameState::WaitForGlass {
                    team,
                    reason: Reason::EarlyStart,
                });
            }
        }
        if current_time > timing {
            machine.play_melody(REACTION_TESTER_WAIT_FOR_REACTION_MELODY);
            Some(GameState::WaitForAllButtonPresses(
                TeamState::new(num_players_left),
                TeamState::new(num_players_right),
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
            Reason::SlowReaction => NUM_SHOTS_SLOW_REACTION,
            Reason::EarlyStart => NUM_SHOTS_EARLY_START,
        };
        let led = match team {
            Team::Left => Led::Left,
            Team::Right => Led::Right,
        };
        if machine.no_ongoing_led_transition() {
            machine.flash_led(led, LED_FLASH_DURATION, LED_ON_DURATION);
        }
        if state.anything_just_pressed() {
            let num_shots = num_shots;
            machine.pour_with_melody(num_shots);
            machine.wait_for_all_actions();
            self.state = get_wait_for_start_state_with_random_timing(machine, &mut self.rng);
        }
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
    players: &[ReactionTesterPlayer],
    team_state: &mut TeamState,
) -> bool {
    for (i, player) in players.iter().enumerate() {
        if state.pressed(Switch::number(player.button_num)) {
            team_state.players_pressed[i] = true;
        }
    }
    team_state.won()
}

impl Program for ReactionTester {
    fn update(&mut self, machine: &mut Machine, state: &State) {
        match self.state {
            GameState::WaitForStart(time) => {
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
