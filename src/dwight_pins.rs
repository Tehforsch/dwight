use rp_pico as bsp;

use bsp::hal::gpio::{
    bank0::{Gpio0, Gpio1, Gpio12, Gpio13, Gpio14, Gpio15},
    DynPinId, FunctionNull, FunctionSioInput, FunctionSioOutput, Pin, Pins, PullDown, PullUp,
};

type NumberSwitchPin = Pin<DynPinId, FunctionSioInput, PullUp>;
type LeftSwitchPin = Pin<Gpio12, FunctionSioInput, PullUp>;
type RightSwitchPin = Pin<Gpio13, FunctionSioInput, PullUp>;

type SpeakerPin = Pin<Gpio1, FunctionNull, PullDown>;
type RelayPin = Pin<Gpio0, FunctionSioOutput, PullDown>;
type LeftLedPin = Pin<Gpio14, FunctionSioOutput, PullDown>;
type RightLedPin = Pin<Gpio15, FunctionSioOutput, PullDown>;

pub struct DwightPins {
    pub number_switches: [NumberSwitchPin; 10],
    pub left_switch: LeftSwitchPin,
    pub right_switch: RightSwitchPin,

    speaker_pin: Option<SpeakerPin>,
    pub relay_pin: RelayPin,
    pub left_led: LeftLedPin,
    pub right_led: RightLedPin,
}

impl DwightPins {
    pub fn new(pins: Pins) -> Self {
        Self {
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
            left_switch: pins.gpio12.into_pull_up_input(),
            right_switch: pins.gpio13.into_pull_up_input(),
            speaker_pin: Some(pins.gpio1),
            relay_pin: pins.gpio0.into_push_pull_output(),
            left_led: pins.gpio14.into_push_pull_output(),
            right_led: pins.gpio15.into_push_pull_output(),
        }
    }

    pub fn speaker_pin(&mut self) -> SpeakerPin {
        self.speaker_pin.take().unwrap()
    }
}
