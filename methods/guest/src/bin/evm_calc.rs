#![no_main]
#![no_std]

extern crate alloc;

use alloc::string::String;
use evm::backend::MemoryVicinity;
use evm_runner::{run_evm, StateMap};
use risc0_zkvm::guest::env;
risc0_zkvm::guest::entry!(main);

pub fn main() {
    let vicinity: MemoryVicinity = env::read();
    let state: StateMap = env::read();
    let input: String = env::read();
    let result = run_evm(&vicinity, state, &input);
    env::commit(&result);
}
