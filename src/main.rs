#![feature(const_fn_floating_point_arithmetic)]
//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

mod tone;

use rp_pico as bsp;

use bsp::{
    entry,
    hal::{self},
};
use defmt_rtt as _;
use embedded_hal::PwmPin;
use panic_probe as _;

use bsp::hal::{
    clocks::{init_clocks_and_plls, Clock},
    pac,
    sio::Sio,
    watchdog::Watchdog,
};
use tone::*;

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

    let mut pwm_slices = hal::pwm::Slices::new(pac.PWM, &mut pac.RESETS);
    let pwm = &mut pwm_slices.pwm0;
    pwm.set_ph_correct();
    pwm.enable();

    pwm.channel_b.output_to(pins.gpio1);
    pwm.set_div_int(80);

    let melody = my_favorite_melody();
    loop {
        for note in melody.iter() {
            note.playback(pwm, &mut delay);

            pwm.channel_b.set_duty(0);
            delay.delay_ms(300);
        }
    }
}
