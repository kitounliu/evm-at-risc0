extern crate alloc;
extern crate core;

use core::str::FromStr;

use alloc::{collections::BTreeMap, string::String, vec::Vec};
use evm_runner::StateMap;
use methods::{EVM_CALC_ID, EVM_CALC_PATH};
use risc0_zkvm::serde::{from_slice, to_vec};
use risc0_zkvm::Prover;

use evm::backend::{MemoryAccount, MemoryVicinity};
use primitive_types::{H160, U256};

use std::time::{Duration, Instant};

pub const CALCULATOR_EVM_PROGRAM: &str = include_str!("../../bytecode/Calculator.bin-runtime");

fn get_input() -> (MemoryVicinity, StateMap) {
    let vicinity = MemoryVicinity {
        gas_price: U256::zero(),
        origin: H160::default(),
        block_hashes: Vec::new(),
        block_number: Default::default(),
        block_coinbase: Default::default(),
        block_timestamp: Default::default(),
        block_difficulty: Default::default(),
        block_gas_limit: Default::default(),
        chain_id: U256::one(),
        block_base_fee_per_gas: U256::zero(),
    };

    let mut state = BTreeMap::new();
    state.insert(
        H160::from_str("0x1000000000000000000000000000000000000000").unwrap(),
        MemoryAccount {
            nonce: U256::one(),
            balance: U256::from(10000000),
            storage: BTreeMap::new(),
            code: hex::decode(CALCULATOR_EVM_PROGRAM).unwrap(),
        },
    );
    state.insert(
        H160::from_str("0xf000000000000000000000000000000000000000").unwrap(),
        MemoryAccount {
            nonce: U256::one(),
            balance: U256::from(10000000),
            storage: BTreeMap::new(),
            code: Vec::new(),
        },
    );
    (vicinity, state)
}

fn run_prover(vicinity: &MemoryVicinity, state: &StateMap, input: &str) -> u32 {
    // Make the prover.
    let method_code = std::fs::read(EVM_CALC_PATH).unwrap();
    let mut prover = Prover::new(&method_code, EVM_CALC_ID).unwrap();

    prover.add_input_u32_slice(to_vec(vicinity).unwrap().as_slice());
    prover.add_input_u32_slice(to_vec(state).unwrap().as_slice());
    prover.add_input_u32_slice(to_vec(input).unwrap().as_slice());

    let now = Instant::now();
    let receipt = prover.run().unwrap();
    println!("proving time {} ms", now.elapsed().as_millis());

    println!("proof size = {}", receipt.get_seal_bytes().len());

    let now = Instant::now();
    assert!(receipt.verify(EVM_CALC_ID).is_ok());
    println!("verifying time {} ms", now.elapsed().as_millis());

    let result: String = from_slice(receipt.journal.as_slice()).unwrap();
    u32::from_str_radix(&result, 16).unwrap()
}

fn main() {
    println!("Proving Calculator.add(7, 2)");

    println!("Loading memory vicinity and state");
    let (vicinity, state) = get_input();

    println!("loading input");
    let input = "771602f700000000000000000000000000000000000000000000000000000000000000070000000000000000000000000000000000000000000000000000000000000002";
    println!("prover starts");
    let result = run_prover(&vicinity, &state, input);
    println!("Proof generated. 7 + 2 = {result}");

    /*
    println!("Proving Calculator.fibonacci(4)");
    let input = "61047ff40000000000000000000000000000000000000000000000000000000000000002";
    let result = run_prover(input);
    println!("Proof generated. fibonacci(4) = {result}");

     */
}
