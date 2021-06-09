#![no_std]
#![feature(trait_alias)]

#[cfg(test)]
#[macro_use]
extern crate std;

extern crate alloc;

#[macro_use]
extern crate num_derive;

mod bcd;
mod ffa;
mod ffb;
mod frameformat;
mod threeoutofsix;
mod mbusaddress;
mod wmbus;
pub mod modec;
pub mod modet;

pub use self::{
    wmbus::WMBusPacket,
    threeoutofsix::ThreeOutOfSix,
};