#![no_std]
#![no_main]

mod dwight_pins;

use bsp::entry;
use bsp::hal::clocks::init_clocks_and_plls;
use bsp::hal::clocks::Clock;
use bsp::hal::pac;
use bsp::hal::pwm::FreeRunning;
use bsp::hal::pwm::Pwm0;
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

const DIVIDER: u8 = 80;

pub struct Dwight {
    pins: DwightPins,
    pwm: Slice<Pwm0, FreeRunning>,
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

        let mut pwm = hal::pwm::Slices::new(pac.PWM, &mut pac.RESETS).pwm0;
        pwm.set_ph_correct();
        pwm.enable();

        pwm.channel_b.output_to(pins.speaker_pin());
        pwm.set_div_int(DIVIDER);
        let start = timer.get_counter();
        Dwight {
            pins,
            pwm,
            delay,
            timer,
            start,
        }
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

    fn set_speaker_frequency(&mut self, freq: &Frequency) {
        if let Frequency::Some(freq) = freq {
            let top = (XTAL_FREQ_HZ as f32 / (DIVIDER as f32 * 0.5) / freq) as u16;
            self.pwm.channel_b.set_duty(top / 2);
            self.pwm.set_top(top);
        } else {
            self.pwm.channel_b.set_duty(0);
        };
    }

    fn wait(&mut self, delay_ms: f32) {
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
