use std::io::{self, BufRead};
use std::sync::mpsc::{channel, Receiver, RecvTimeoutError};
use std::thread::JoinHandle;
use std::{thread, time::Duration};

use dwight::{
    machine::{Frequency, Led, LedState, Machine, RelayState, Switch, SwitchState},
    main_loop,
};

pub const RECV_TIMEOUT_MS: u64 = 5;

struct TestDwight {
    input_reader: InputReader,
}

impl TestDwight {
    pub fn new() -> Self {
        Self {
            input_reader: InputReader::new(),
        }
    }
}

impl Machine for TestDwight {
    fn get_switch_state(&mut self, switch: Switch) -> SwitchState {
        if let Some(input) = self.input_reader.next_input() {
            let input_switch = input_to_switch(&input);
            if let Some(input_switch) = input_switch {
                if input_switch == switch {
                    return SwitchState::Pressed;
                }
            }
        }
        SwitchState::Released
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
        .map(|num| Switch::Number(num))
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
    let dwight = TestDwight::new();
    main_loop(dwight);
}
