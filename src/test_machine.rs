use std::io::{self, BufRead};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError};
use std::thread::JoinHandle;
use std::time::Instant;
use std::{thread, time::Duration};

use dwight::hardware_interface::{
    Frequency, HardwareInterface, Led, LedState, RelayState, Switch, SwitchState,
};
use dwight::{Machine, SimplePouring};

pub const RECV_TIMEOUT_MS: u64 = 5;

pub const PRESSED_DURATION_MS: u128 = 200;

struct TestDwight {
    input_reader: InputReader,
    pressed: Vec<(Instant, Switch)>,
}

impl TestDwight {
    pub fn new() -> Self {
        Self {
            input_reader: InputReader::new(),
            pressed: vec![],
        }
    }
}

impl TestDwight {
    fn update_switches(&mut self) {
        let now = Instant::now();
        while let Some(switch) = self
            .input_reader
            .next_input()
            .and_then(|input| input_to_switch(&input))
        {
            self.pressed.push((now, switch));
        }
        self.pressed = self
            .pressed
            .drain(..)
            .filter_map(|(instant, switch)| {
                if now.duration_since(instant).as_millis() > PRESSED_DURATION_MS {
                    None
                } else {
                    Some((instant, switch))
                }
            })
            .collect();
    }
}

impl HardwareInterface for TestDwight {
    fn get_switch_state(&mut self, switch: Switch) -> SwitchState {
        self.update_switches();
        if self.pressed.iter().any(|(_, pressed)| pressed == &switch) {
            SwitchState::Pressed
        } else {
            SwitchState::Released
        }
    }

    fn set_led_state(&mut self, led: Led, led_state: LedState) {
        dbg!(led, led_state);
    }

    fn set_relay_state(&mut self, relay_state: RelayState) {
        dbg!(relay_state);
    }

    fn play_frequency(&mut self, frequency: &Frequency) {
        dbg!(frequency);
    }

    fn wait(&mut self, delay_ms: f32) {
        thread::sleep(Duration::from_millis(delay_ms as u64));
    }
}

fn input_to_switch(input: &str) -> Option<Switch> {
    input
        .parse::<usize>()
        .ok()
        .map(|num| Switch::number(num))
        .or_else(|| {
            if input == "left" {
                Some(Switch::Left)
            } else if input == "right" {
                Some(Switch::Left)
            } else {
                None
            }
        })
}

struct InputReader {
    receiver: Receiver<String>,
    // We can never join since the mainloop returns !
    _handle: JoinHandle<()>,
}

impl InputReader {
    fn new() -> Self {
        let (sender, receiver) = channel();
        let _handle = thread::spawn(move || loop {
            let stdin = io::stdin();
            for line in stdin.lock().lines() {
                let line = line.unwrap();
                sender.send(line).unwrap();
            }
        });
        Self { receiver, _handle }
    }

    fn next_input(&mut self) -> Option<String> {
        self.receiver
            .recv_timeout(Duration::from_millis(RECV_TIMEOUT_MS))
            .map_err(|err| {
                if err == RecvTimeoutError::Disconnected {
                    panic!("Receiver disconnected")
                }
            })
            .ok()
    }
}

fn main() {
    let dwight_interface = TestDwight::new();
    let program = SimplePouring;
    let machine = Machine::new(program, dwight_interface);
    machine.run();
}
