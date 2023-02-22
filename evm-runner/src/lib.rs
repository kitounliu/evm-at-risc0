#![cfg_attr(not(test), no_std)]

extern crate alloc;
extern crate core;

use alloc::{collections::BTreeMap, string::String, vec::Vec};
use core::str::FromStr;
use evm::{
    backend::{ApplyBackend, MemoryAccount, MemoryBackend, MemoryVicinity},
    executor::stack::{MemoryStackState, StackExecutor, StackSubstateMetadata},
    Config, ExitReason, ExitSucceed,
};
use primitive_types::{H160, U256};

pub type StateMap = BTreeMap<H160, MemoryAccount>;

//mod transaction;

pub fn run_evm(vicinity: &MemoryVicinity, state: StateMap, input: &str) -> String {
    let config = Config::istanbul();

    let mut backend = MemoryBackend::new(vicinity, state);
    let metadata = StackSubstateMetadata::new(u64::MAX, &config);
    let state = MemoryStackState::new(metadata, &backend);
    let precompiles = BTreeMap::new();
    let mut executor = StackExecutor::new_with_precompiles(state, &config, &precompiles);

    let (exit_reason, result) = executor.transact_call(
        H160::from_str("0xf000000000000000000000000000000000000000").unwrap(),
        H160::from_str("0x1000000000000000000000000000000000000000").unwrap(),
        U256::zero(),
        hex::decode(input).unwrap(),
        u64::MAX,
        Vec::new(),
    );
    assert!(exit_reason == ExitReason::Succeed(ExitSucceed::Returned));

    let (values, logs) = executor.into_state().deconstruct();
    backend.apply(values, logs, delete_empty);

    hex::encode(result)
}

#[cfg(test)]
mod tests {
    use super::*;

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

    #[test]
    fn evm_calc_works() {
        let (vicinity, state) = get_input();
        let data = "771602f700000000000000000000000000000000000000000000000000000000000000070000000000000000000000000000000000000000000000000000000000000002";
        let result = run_evm(&vicinity, state, data);
        assert_eq!(
            result,
            "0000000000000000000000000000000000000000000000000000000000000009"
        );
    }
}
