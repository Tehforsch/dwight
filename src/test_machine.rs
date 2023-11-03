use std::{thread, time::Duration};

use dwight::{
    machine::{Frequency, Led, LedState, Machine, RelayState, Switch, SwitchState},
    main_loop,
};

struct TestDwight {}

impl Machine for TestDwight {
    fn get_switch_state(&mut self, _switch: Switch) -> SwitchState {
        SwitchState::Pressed
    }

    fn set_led_state(&mut self, led: &Led, led_state: &LedState) {
        dbg!(led, led_state);
    }

    fn set_relay_state(&mut self, relay_state: &RelayState) {
        dbg!(relay_state);
    }

    fn play_frequency(&mut self, frequency: &Frequency) {
        dbg!(frequency);
    }

    fn wait(&mut self, delay_ms: f32) {
        thread::sleep(Duration::from_millis(delay_ms as u64));
    }
}

fn main() {
    let dwight = TestDwight {};
    main_loop(dwight);
}
