#![cfg(feature = "test-utils")]

use crate::env_exports::Timestamp;
use ink_env::{
    test::{ChainSpec, EmittedEvent},
    AccountId, DefaultEnvironment,
};

/// Fast way to get default_accounts
pub fn default_accounts() -> ink_env::test::DefaultAccounts<DefaultEnvironment> {
    ink_env::test::default_accounts().expect("could not get default accounts")
}

/// Default caller of the contract that is used when testing
pub fn default_caller() -> AccountId { default_accounts().alice }

/// Get the last emitted event
pub fn last_event() -> Option<EmittedEvent> { ink_env::test::recorded_events().last() }

/// The number of recorded events
pub fn recorded_event_count() -> usize { ink_env::test::recorded_events().count() }

/// Sets the caller for the next call in a test function
pub fn set_caller(caller: AccountId) {
    let callee = ink_env::account_id::<DefaultEnvironment>().unwrap_or([0x0; 32].into());
    ink_env::test::push_execution_context::<DefaultEnvironment>(
        caller,
        callee,
        1000000,
        1000000,
        ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4])), // dummy
    );
}

/// Get the block time
pub fn block_time() -> Timestamp {
    let mut chain_spec = ChainSpec::uninitialized();
    chain_spec.initialize_as_default::<DefaultEnvironment>().unwrap();
    chain_spec.block_time::<DefaultEnvironment>().unwrap()
}

/// Advances time in milliseconds
pub fn advance_time(millis: Timestamp) {
    let block_time = block_time();
    let block_count = millis / block_time;

    for _ in 0..block_count {
        ink_env::test::advance_block::<DefaultEnvironment>().unwrap();
    }
}
