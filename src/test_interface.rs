use std::io::BufRead;
use std::io::{self};
use std::sync::mpsc::channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::RecvTimeoutError;
use std::thread;
use std::thread::JoinHandle;
use std::time::Duration;
use std::time::Instant;

use dwight::hardware_interface::Frequency;
use dwight::hardware_interface::HardwareInterface;
use dwight::hardware_interface::Led;
use dwight::hardware_interface::LedState;
use dwight::hardware_interface::RelayState;
use dwight::hardware_interface::Switch;
use dwight::hardware_interface::SwitchState;
use dwight::main_loop;
use dwight::Time;

pub const RECV_TIMEOUT_MS: u64 = 5;
pub const PRESSED_DURATION_MS: u128 = 1000;

struct TestDwight {
    input_reader: InputReader,
    pressed: Vec<(Instant, Switch)>,
    start: Instant,
}

impl TestDwight {
    pub fn new() -> Self {
        Self {
            input_reader: InputReader::new(),
            pressed: vec![],
            start: Instant::now(),
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

    fn set_led_state(&mut self, _led: Led, _led_state: LedState) {}

    fn set_relay_state(&mut self, relay_state: RelayState) {
        dbg!(relay_state);
    }

    fn set_speaker_frequency(&mut self, frequency: &Frequency) {
        dbg!(frequency);
    }

    fn wait_ms(&mut self, delay_ms: dwight::Duration) {
        thread::sleep(Duration::from_millis(delay_ms as u64));
    }

    fn get_elapsed_time_ms(&mut self) -> Time {
        Instant::now().duration_since(self.start).as_millis() as Time
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
                Some(Switch::Right)
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
    main_loop(TestDwight::new())
}
