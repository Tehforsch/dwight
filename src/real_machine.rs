#![no_std]
#![no_main]

mod dwight_pins;

use cortex_m::delay::Delay;
use dwight_pins::DwightPins;
use embedded_hal::{
    digital::v2::{InputPin, OutputPin},
    PwmPin,
};
use rp_pico as bsp;

use bsp::{
    entry,
    hal::{
        self,
        pwm::{FreeRunning, Pwm0, Slice},
    },
};
use defmt_rtt as _;
use panic_probe as _;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};

use dwight::{
    hardware_interface::{
        Frequency, HardwareInterface, Led, LedState, RelayState, Switch, SwitchState,
    },
    Machine, SimplePouring,
};

/// External high-speed crystal on the Raspberry Pi Pico board is 12 MHz. Adjust
/// if your board has a different frequency
pub const XTAL_FREQ_HZ: u32 = 12_000_000u32;

const DIVIDER: i32 = 40;

pub struct Dwight {
    pins: DwightPins,
    pwm: Slice<Pwm0, FreeRunning>,
    delay: Delay,
}

impl Dwight {
    fn new() -> Self {
        let mut pac = pac::Peripherals::take().unwrap();
        let core = pac::CorePeripherals::take().unwrap();
        let mut watchdog = Watchdog::new(pac.WATCHDOG);
        let sio = Sio::new(pac.SIO);

        let clocks = init_clocks_and_plls(
            XTAL_FREQ_HZ,
            pac.XOSC,
            pac.CLOCKS,
            pac.PLL_SYS,
            pac.PLL_USB,
            &mut pac.RESETS,
            &mut watchdog,
        )
        .ok()
        .unwrap();

        let delay = Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

        let pins = hal::gpio::Pins::new(
            pac.IO_BANK0,
            pac.PADS_BANK0,
            sio.gpio_bank0,
            &mut pac.RESETS,
        );
        let mut pins = DwightPins::new(pins);

        let mut pwm = hal::pwm::Slices::new(pac.PWM, &mut pac.RESETS).pwm0;
        pwm.set_ph_correct();
        pwm.enable();

        pwm.channel_b.output_to(pins.speaker_pin());
        pwm.set_div_int(80);
        Dwight { pins, pwm, delay }
    }
}

impl HardwareInterface for Dwight {
    fn get_switch_state(&mut self, switch: Switch) -> SwitchState {
        let is_pressed = match switch {
            Switch::Number0 => self.pins.number_switches[0].is_low(),
            Switch::Number1 => self.pins.number_switches[1].is_low(),
            Switch::Number2 => self.pins.number_switches[2].is_low(),
            Switch::Number3 => self.pins.number_switches[3].is_low(),
            Switch::Number4 => self.pins.number_switches[4].is_low(),
            Switch::Number5 => self.pins.number_switches[5].is_low(),
            Switch::Number6 => self.pins.number_switches[6].is_low(),
            Switch::Number7 => self.pins.number_switches[7].is_low(),
            Switch::Number8 => self.pins.number_switches[8].is_low(),
            Switch::Number9 => self.pins.number_switches[9].is_low(),
            Switch::Left => self.pins.left_switch.is_low(),
            Switch::Right => self.pins.right_switch.is_low(),
        }
        .unwrap();
        if is_pressed {
            SwitchState::Pressed
        } else {
            SwitchState::Released
        }
    }

    fn set_led_state(&mut self, led: Led, led_state: LedState) {
        match (led_state, led) {
            (LedState::On, Led::Left) => self.pins.left_led.set_high(),
            (LedState::On, Led::Right) => self.pins.right_led.set_high(),
            (LedState::Off, Led::Left) => self.pins.left_led.set_low(),
            (LedState::Off, Led::Right) => self.pins.right_led.set_low(),
        }
        .unwrap();
    }

    fn set_relay_state(&mut self, relay_state: RelayState) {
        match relay_state {
            RelayState::On => self.pins.relay_pin.set_high(),
            RelayState::Off => self.pins.relay_pin.set_low(),
        }
        .unwrap();
    }

    fn play_frequency(&mut self, freq: &Frequency) {
        if let Frequency::Some(freq) = freq {
            let top = (XTAL_FREQ_HZ as f32 / DIVIDER as f32 / freq) as u16;
            self.pwm.channel_b.set_duty(top / 2);
            self.pwm.set_top(top);
        } else {
            self.pwm.channel_b.set_duty(0);
        };
    }

    fn wait(&mut self, delay_ms: f32) {
        self.delay.delay_ms(delay_ms as u32);
    }
}

#[entry]
fn main() -> ! {
    let dwight_interface = Dwight::new();
    let program = SimplePouring;
    let machine = Machine::new(program, dwight_interface);
    machine.run();
}
