#![no_std]
#![no_main]

mod dwight_pins;

use bsp::entry;
use bsp::hal::clocks::init_clocks_and_plls;
use bsp::hal::clocks::Clock;
use bsp::hal::pac;
use bsp::hal::pwm::FreeRunning;
use bsp::hal::pwm::Pwm0;
use bsp::hal::pwm::Pwm7;
use bsp::hal::pwm::Slice;
use bsp::hal::sio::Sio;
use bsp::hal::timer::Instant;
use bsp::hal::watchdog::Watchdog;
use bsp::hal::Timer;
use bsp::hal::{self};
use cortex_m::delay::Delay;
use defmt_rtt as _;
use dwight::hardware_interface::Frequency;
use dwight::hardware_interface::HardwareInterface;
use dwight::hardware_interface::Led;
use dwight::hardware_interface::LedState;
use dwight::hardware_interface::RelayState;
use dwight::hardware_interface::Switch;
use dwight::hardware_interface::SwitchState;
use dwight::main_loop;
use dwight::Duration;
use dwight::Time;
use dwight_pins::DwightPins;
use embedded_alloc::Heap;
use embedded_hal::digital::v2::InputPin;
use embedded_hal::digital::v2::OutputPin;
use embedded_hal::PwmPin;
use panic_probe as _;
use rp_pico as bsp;

#[global_allocator]
static HEAP: Heap = Heap::empty();

/// External high-speed crystal on the Raspberry Pi Pico board is 12 MHz. Adjust
/// if your board has a different frequency
pub const XTAL_FREQ_HZ: u32 = 12_000_000u32;

const AUDIO_PWM_DIVIDER: u8 = 80;

/// Value from the Raspberry Pi Pico hal pwm_blink template.
pub const DEFAULT_LED_TOP: f32 = 25000.0;

pub struct Dwight {
    pins: DwightPins,
    audio_pwm: Slice<Pwm0, FreeRunning>,
    led_pwm: Slice<Pwm7, FreeRunning>,
    delay: Delay,
    timer: Timer,
    start: Instant,
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
        let timer = Timer::new(pac.TIMER, &mut pac.RESETS, &clocks);

        let pins = hal::gpio::Pins::new(
            pac.IO_BANK0,
            pac.PADS_BANK0,
            sio.gpio_bank0,
            &mut pac.RESETS,
        );
        let mut pins = DwightPins::new(pins);

        let pwm_slices = hal::pwm::Slices::new(pac.PWM, &mut pac.RESETS);
        let mut audio_pwm = pwm_slices.pwm0;
        audio_pwm.set_ph_correct();
        audio_pwm.enable();
        audio_pwm.channel_b.output_to(pins.speaker_pin());
        audio_pwm.set_div_int(AUDIO_PWM_DIVIDER);

        let mut led_pwm = pwm_slices.pwm7;
        led_pwm.set_ph_correct();
        led_pwm.enable();
        let (left_led, right_led) = pins.led_pins();
        led_pwm.channel_a.output_to(left_led);
        led_pwm.channel_b.output_to(right_led);
        led_pwm.set_top(DEFAULT_LED_TOP as u16);

        let start = timer.get_counter();
        Dwight {
            pins,
            audio_pwm,
            led_pwm,
            delay,
            timer,
            start,
        }
    }
}

fn brightness_to_voltage(brightness: f32) -> f32 {
    const NUM_ENTRIES: usize = 64;
    let lookup: [f32; NUM_ENTRIES] = [
        0.0,
        0.0009765625,
        0.0009765625,
        0.001953125,
        0.001953125,
        0.001953125,
        0.001953125,
        0.001953125,
        0.0029296875,
        0.0029296875,
        0.0029296875,
        0.00390625,
        0.00390625,
        0.0048828125,
        0.0048828125,
        0.005859375,
        0.005859375,
        0.0068359375,
        0.0078125,
        0.0087890625,
        0.009765625,
        0.0107421875,
        0.01171875,
        0.0126953125,
        0.0146484375,
        0.0166015625,
        0.0185546875,
        0.0205078125,
        0.0224609375,
        0.025390625,
        0.0283203125,
        0.03125,
        0.03515625,
        0.0390625,
        0.04296875,
        0.0478515625,
        0.0537109375,
        0.0595703125,
        0.06640625,
        0.07421875,
        0.0830078125,
        0.091796875,
        0.1025390625,
        0.1142578125,
        0.1279296875,
        0.142578125,
        0.158203125,
        0.1767578125,
        0.197265625,
        0.2197265625,
        0.244140625,
        0.2724609375,
        0.3037109375,
        0.337890625,
        0.376953125,
        0.419921875,
        0.4677734375,
        0.521484375,
        0.5810546875,
        0.6474609375,
        0.7216796875,
        0.8046875,
        0.896484375,
        0.9990234375,
    ];
    let index = ((brightness * NUM_ENTRIES as f32) as usize).clamp(0, NUM_ENTRIES - 1);
    lookup[index]
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
        let brightness_factor = led_state.brightness;

        let voltage_factor = brightness_to_voltage(brightness_factor);
        match led {
            Led::Left => self
                .led_pwm
                .channel_a
                .set_duty((DEFAULT_LED_TOP * voltage_factor) as u16),

            Led::Right => self
                .led_pwm
                .channel_b
                .set_duty((DEFAULT_LED_TOP * voltage_factor) as u16),
        };
    }

    fn set_relay_state(&mut self, relay_state: RelayState) {
        match relay_state {
            RelayState::On => self.pins.relay_pin.set_high(),
            RelayState::Off => self.pins.relay_pin.set_low(),
        }
        .unwrap();
    }

    fn set_speaker_frequency(&mut self, freq: &Frequency) {
        if let Frequency::Some(freq) = freq {
            let top = (XTAL_FREQ_HZ as f32 / (AUDIO_PWM_DIVIDER as f32 * 0.5) / freq) as u16;
            self.audio_pwm.channel_b.set_duty(top / 2);
            self.audio_pwm.set_top(top);
        } else {
            self.audio_pwm.channel_b.set_duty(0);
        };
    }

    fn wait_ms(&mut self, delay_ms: Duration) {
        self.delay.delay_ms(delay_ms as u32);
    }

    fn get_elapsed_time_ms(&mut self) -> Time {
        self.timer
            .get_counter()
            .checked_duration_since(self.start)
            .unwrap()
            .to_millis() as Time
    }
}

fn init_allocator() {
    use core::mem::MaybeUninit;
    const HEAP_SIZE: usize = 65536;
    static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
    unsafe { HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE) }
}

#[entry]
fn main() -> ! {
    init_allocator();
    main_loop(Dwight::new())
}
