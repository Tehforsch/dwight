use rp_pico as bsp;

use bsp::hal::gpio::{
    bank0::Gpio1, DynPinId, FunctionNull, FunctionSioInput, Pin, Pins,
    PullDown, PullUp,
};

type NumberSwitch = Pin<DynPinId, FunctionSioInput, PullUp>;
type SpeakerPin = Pin<Gpio1, FunctionNull, PullDown>;

pub struct DwightPins {
    pub speaker_pin: Option<SpeakerPin>,
    number_switches: [NumberSwitch; 10],
}

impl DwightPins {
    pub fn new(pins: Pins) -> Self {
        Self {
            speaker_pin: Some(pins.gpio1),
            number_switches: [
                // This is the 0 pin, so it comes first
                pins.gpio11.into_pull_up_input().into_dyn_pin(),
                pins.gpio2.into_pull_up_input().into_dyn_pin(),
                pins.gpio3.into_pull_up_input().into_dyn_pin(),
                pins.gpio4.into_pull_up_input().into_dyn_pin(),
                pins.gpio5.into_pull_up_input().into_dyn_pin(),
                pins.gpio6.into_pull_up_input().into_dyn_pin(),
                pins.gpio7.into_pull_up_input().into_dyn_pin(),
                pins.gpio8.into_pull_up_input().into_dyn_pin(),
                pins.gpio9.into_pull_up_input().into_dyn_pin(),
                pins.gpio10.into_pull_up_input().into_dyn_pin(),
            ],
        }
    }

    pub fn speaker_pin(&mut self) -> SpeakerPin {
        self.speaker_pin.take().unwrap()
    }

    pub fn iter_number_switches(&self) -> impl Iterator<Item = (usize, &NumberSwitch)> {
        self.number_switches.iter().enumerate()
    }
}
