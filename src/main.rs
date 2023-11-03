#![no_std]
#![no_main]

mod dwight_pins;
mod melody;

use dwight_pins::DwightPins;
use embedded_hal::digital::v2::{InputPin};
use rp_pico as bsp;

use bsp::{
    entry,
    hal::{self},
};
use defmt_rtt as _;
use panic_probe as _;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};
use melody::*;

#[entry]
fn main() -> ! {
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

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());

    let pins = hal::gpio::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );
    let mut pins = DwightPins::new(pins);

    let mut pwm_slices = hal::pwm::Slices::new(pac.PWM, &mut pac.RESETS);
    let pwm = &mut pwm_slices.pwm0;
    pwm.set_ph_correct();
    pwm.enable();

    pwm.channel_b.output_to(pins.speaker_pin());
    pwm.set_div_int(80);

    let melody = beethoven_9();
    for note in melody.iter() {
        note.playback(pwm, &mut delay);
    }

    loop {
        for num in 0..10 {
            if pins.get_number_switch(num).is_low().unwrap() {
                for _ in 0..num {
                    Note {
                        freq: Note::A4,
                        length: Length::Eighth,
                    }
                    .playback(pwm, &mut delay);
                }
            }
        }
    }
}
