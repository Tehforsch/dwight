#![feature(const_fn_floating_point_arithmetic)]
//! Blinks the LED on a Pico board
//!
//! This will blink an LED attached to GP25, which is the pin the Pico uses for the on-board LED.
#![no_std]
#![no_main]

mod tone;

use bsp::{
    entry,
    hal::pwm::{InputHighRunning, Slices},
};
use defmt_rtt as _;
use embedded_hal::{digital::v2::OutputPin, PwmPin};
use panic_probe as _;

// Provide an alias for our BSP so we can switch targets quickly.
// Uncomment the BSP you included in Cargo.toml, the rest of the code does not need to change.
use rp_pico as bsp;
// use sparkfun_pro_micro_rp2040 as bsp;

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

    // External high-speed crystal on the pico board is 12Mhz
    let external_xtal_freq_hz = 12_000_000u32;
    let clocks = init_clocks_and_plls(
        external_xtal_freq_hz,
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

    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    // This is the correct pin on the Raspberry Pico board. On other boards, even if they have an
    // on-board LED, it might need to be changed.
    // Notably, on the Pico W, the LED is not connected to any of the RP2040 GPIOs but to the cyw43 module instead. If you have
    // a Pico W and want to toggle a LED with a simple GPIO output pin, you can connect an external
    // LED to one of the GPIO pins, and reference that pin here.
    let mut led_pin = pins.led.into_push_pull_output();

    // Init PWMs
    let pwm_slices = Slices::new(pac.PWM, &mut pac.RESETS);

    // Configure PWM4
    let mut pwm = pwm_slices.pwm0;
    pwm.set_ph_correct();
    pwm.enable();

    // Set to run when b channel is high
    let mut pwm = pwm.into_mode::<InputHighRunning>();
    pwm.channel_b.input_from(pins.gpio1);

    let twinkle_twinkle = [
        C4, C4, G4, G4, A4, A4, G4, NO_NOTE, F4, F4, E4, E4, D4, D4, C4, NO_NOTE, G4, G4, F4, F4,
        E4, E4, D4, NO_NOTE, G4, G4, F4, F4, E4, E4, D4, NO_NOTE, C4, C4, G4, G4, A4, A4, G4,
        NO_NOTE, F4, F4, E4, E4, D4, D4, C4, NO_NOTE,
    ];

    for top in twinkle_twinkle {
        pwm.channel_b.set_duty(top / 2); // 50% Duty Cycle
        pwm.set_top(top);
        delay.delay_ms(500);

        pwm.channel_b.set_duty(0);
        delay.delay_ms(500);
    }

    loop {
        led_pin.set_high().unwrap();
        delay.delay_ms(500);
        led_pin.set_low().unwrap();
        delay.delay_ms(500);
    }
}
