use enum_map::Enum;
use enum_map::EnumMap;

use crate::Duration;
use crate::Time;

#[derive(Debug)]
pub struct LedState {
    pub brightness: f32,
}
impl LedState {
    pub fn on() -> LedState {
        Self { brightness: 0.0 }
    }

    pub fn off() -> LedState {
        Self { brightness: 0.0 }
    }
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
    pub const C_SHARP_4: Frequency = Frequency::Some(277.18);
    pub const D4: Frequency = Frequency::Some(293.66);
    pub const D_SHARP_4: Frequency = Frequency::Some(311.13);
    pub const E4: Frequency = Frequency::Some(329.63);
    pub const F4: Frequency = Frequency::Some(349.23);
    pub const F_SHARP_4: Frequency = Frequency::Some(369.99);
    pub const G4: Frequency = Frequency::Some(392.00);
    pub const G_SHARP_4: Frequency = Frequency::Some(415.30);
    pub const A4: Frequency = Frequency::Some(440.00);
    pub const A_SHARP_4: Frequency = Frequency::Some(466.16);
    pub const B4: Frequency = Frequency::Some(493.88);
    pub const BREAK: Frequency = Frequency::Silence;
    pub const C5: Frequency = Frequency::Some(523.26);
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

    pub fn get_num(&self) -> Option<usize> {
        match self {
            Switch::Number0 => Some(0),
            Switch::Number1 => Some(1),
            Switch::Number2 => Some(2),
            Switch::Number3 => Some(3),
            Switch::Number4 => Some(4),
            Switch::Number5 => Some(5),
            Switch::Number6 => Some(6),
            Switch::Number7 => Some(7),
            Switch::Number8 => Some(8),
            Switch::Number9 => Some(9),
            Switch::Left => None,
            Switch::Right => None,
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

    pub fn just_pressed(&self, switch: Switch) -> bool {
        self.current[switch].is_pressed() && !self.previous[switch].is_pressed()
    }

    pub fn pressed(&self, switch: Switch) -> bool {
        self.current[switch].is_pressed()
    }

    pub fn iter_just_pressed(&self) -> impl Iterator<Item = Switch> + '_ {
        self.current
            .iter()
            .map(|(switch, _)| switch)
            .filter(|switch| self.just_pressed(*switch))
    }

    pub fn iter_pressed(&self) -> impl Iterator<Item = Switch> + '_ {
        self.current
            .iter()
            .map(|(switch, _)| switch)
            .filter(|switch| self.pressed(*switch))
    }

    pub fn anything_just_pressed(&self) -> bool {
        self.iter_just_pressed().count() > 0
    }

    pub fn anything_pressed(&self) -> bool {
        self.iter_pressed().count() > 0
    }

    pub fn lowest_pressed_number_key(&self) -> Option<usize> {
        self.iter_just_pressed()
            .filter_map(|switch| switch.get_num())
            .next()
    }
}

#[derive(Debug)]
pub enum Action {
    SetLedState(Led, LedState),
    SetRelayState(RelayState),
    SetSpeakerFrequency(Frequency),
}

pub trait HardwareInterface {
    fn get_switch_state(&mut self, switch: Switch) -> SwitchState;
    fn set_led_state(&mut self, led: Led, led_state: LedState);
    fn set_relay_state(&mut self, relay_state: RelayState);
    fn set_speaker_frequency(&mut self, frequency: &Frequency);
    fn wait_ms(&mut self, delay_ms: Duration);
    fn get_elapsed_time_ms(&mut self) -> Time;

    fn perform_action(&mut self, action: Action) {
        match action {
            Action::SetLedState(led, state) => self.set_led_state(led, state),
            Action::SetRelayState(state) => self.set_relay_state(state),
            Action::SetSpeakerFrequency(freq) => self.set_speaker_frequency(&freq),
        }
    }

    fn update_state(&mut self, previous: State) -> State {
        previous.update(EnumMap::from_fn(|switch| self.get_switch_state(switch)))
    }
}
