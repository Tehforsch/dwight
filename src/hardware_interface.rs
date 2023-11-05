use enum_map::Enum;
use enum_map::EnumMap;

use crate::Time;

#[derive(Debug)]
pub enum LedState {
    On,
    Off,
}

#[derive(Debug, Clone, Copy)]
pub enum Led {
    Left,
    Right,
}

#[derive(Debug)]
pub enum RelayState {
    On,
    Off,
}

#[derive(Debug, Clone)]
pub enum Frequency {
    Some(f32),
    Silence,
}

impl Frequency {
    pub const C4: Frequency = Frequency::Some(261.63);
    pub const D4: Frequency = Frequency::Some(293.66);
    pub const E4: Frequency = Frequency::Some(329.63);
    pub const F4: Frequency = Frequency::Some(349.23);
    pub const G4: Frequency = Frequency::Some(392.00);
    pub const A4: Frequency = Frequency::Some(440.00);
    pub const B4: Frequency = Frequency::Some(493.88);
    pub const BREAK: Frequency = Frequency::Silence;
}

#[derive(Debug, PartialEq, Enum, Clone, Copy)]
pub enum Switch {
    // Enumerate these explicitly here instead of doing
    // Number(usize), so that we can build an enum map
    // for convenience later.
    Number0,
    Number1,
    Number2,
    Number3,
    Number4,
    Number5,
    Number6,
    Number7,
    Number8,
    Number9,
    Left,
    Right,
}

impl Switch {
    pub fn number(num: usize) -> Self {
        use Switch::*;
        match num {
            0 => Number0,
            1 => Number1,
            2 => Number2,
            3 => Number3,
            4 => Number4,
            5 => Number5,
            6 => Number6,
            7 => Number7,
            8 => Number8,
            9 => Number9,
            _ => panic!(),
        }
    }
}

#[derive(Debug)]
pub enum SwitchState {
    Pressed,
    Released,
}

impl SwitchState {
    fn is_pressed(&self) -> bool {
        match self {
            SwitchState::Pressed => true,
            SwitchState::Released => false,
        }
    }
}

pub struct State {
    current: EnumMap<Switch, SwitchState>,
    previous: EnumMap<Switch, SwitchState>,
}

impl State {
    pub fn new() -> Self {
        Self {
            current: EnumMap::from_fn(|_| SwitchState::Released),
            previous: EnumMap::from_fn(|_| SwitchState::Released),
        }
    }

    fn update(self, new: EnumMap<Switch, SwitchState>) -> Self {
        Self {
            current: new,
            previous: self.current,
        }
    }
}

impl State {
    pub fn just_pressed(&self, switch: Switch) -> bool {
        self.current[switch].is_pressed() && !self.previous[switch].is_pressed()
    }
}

pub enum HardwareAction {
    SetLedState(Led, LedState),
    SetRelayState(RelayState),
    PlayFrequency(Frequency),
}

pub trait HardwareInterface {
    fn get_switch_state(&mut self, switch: Switch) -> SwitchState;
    fn set_led_state(&mut self, led: Led, led_state: LedState);
    fn set_relay_state(&mut self, relay_state: RelayState);
    fn play_frequency(&mut self, frequency: &Frequency);
    fn wait(&mut self, delay_ms: f32);
    fn get_elapsed_time_ms(&mut self) -> Time;

    fn perform_action(&mut self, action: HardwareAction) {
        match action {
            HardwareAction::SetLedState(led, state) => self.set_led_state(led, state),
            HardwareAction::SetRelayState(state) => self.set_relay_state(state),
            HardwareAction::PlayFrequency(freq) => self.play_frequency(&freq),
        }
    }

    fn update_state(&mut self, previous: State) -> State {
        previous.update(EnumMap::from_fn(|switch| self.get_switch_state(switch)))
    }
}
