// #![no_std]
#![feature(const_fn_floating_point_arithmetic)]

extern crate alloc;

use hardware_interface::HardwareInterface;
use machine::Machine;
use programs::ProgramSwitching;
mod configuration;
pub mod hardware_interface;
mod machine;
mod melody;
mod programs;
mod reaction_tester;

pub type Time = u32;
pub type Duration = u32;

pub fn main_loop(interface: impl HardwareInterface) -> ! {
    Machine::new().run(interface, ProgramSwitching::default())
}
