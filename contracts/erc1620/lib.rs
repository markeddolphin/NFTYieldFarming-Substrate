#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unused_must_use)]

pub use contract::{Erc1620, Stream, StreamId};

use ink_lang as ink;
use scale::{Decode, Encode};


/// Error types
#[derive(Debug, Copy, Clone, PartialEq, Eq, Encode, Decode, err_derive::Error)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    /// Only callable by sender or recipient
    #[error(display = "only callable by sender or recipient")]
    OnlyCallableBySenderOrRecipient,
    /// The balance is insufficient
    #[error(display = "the balance is insufficient")]
    InsufficientBalance,
    /// The amount cannot be zero
    #[error(display = "the amount cannot be zero")]
    AmountCannotBeZero,
    /// The stream was not found
    #[error(display = "the stream was not found")]
    StreamNotFound,
    /// Deposit not multiple of zero
    #[error(display = "deposit not multiple of zero")]
    DepositNotMultipleOfZero,
    /// Deposit smaller than time delta
    #[error(display = "deposit smaller than time delta")]
    DepositSmallerThanTimeDelta,
    /// The recipient is invalid
    #[error(display = "The recipient is invalid")]
    InvalidRecipient,
    /// The start time is invalid
    #[error(display = "The start time is invalid")]
    InvalidStartTime,
    /// The stop time is invalid
    #[error(display = "The stop time is invalid")]
    InvalidStopTime,
    /// An ERC-20 error occurred
    #[error(display = "An Erc20 error occurred: {}", _0)]
    Erc20(#[source] erc20::Error),
    /// Indicates that the account id is not the recipient of any active streams.
    #[error(display = "account has no active streams")]
    StreamsNotFound,
}

/// The result type.
pub type Result<T> = core::result::Result<T, Error>;

#[allow(clippy::enum_variant_names)]
#[ink::contract]
mod contract {
    use super::*;
    #[cfg(not(feature = "ink-as-dependency"))]
    use contract_utils::ZERO_ACCOUNT;
    use erc20::Erc20;
    use ink_env::call::FromAccountId;
    use ink_prelude::{vec, vec::Vec};

    /// An ERC-1620 contract
    #[ink(storage)]
    pub struct Erc1620 {
        /// Map of streams by id
        streams_by_id: ink_storage::collections::HashMap<StreamId, Stream>,
        /// The next [StreamId]
        next_stream_id: ink_storage::lazy::Lazy<StreamId>,
        stream_ids_by_account: ink_storage::collections::HashMap<AccountId, Vec<StreamId>>,
    }

    // Events

    /// Event emitted when a [Stream] is created
    #[ink(event)]
    pub struct CreateStream {
        #[ink(topic)]
        pub stream_id: StreamId,
        #[ink(topic)]
        pub sender: AccountId,
        #[ink(topic)]
        pub recipient: AccountId,
        pub deposit: Balance,
        pub token_address: AccountId,
        pub start_time: Timestamp,
        pub stop_time: Timestamp,
    }

    /// Event emitted when stream is withdrawn from
    #[ink(event)]
    pub struct WithdrawFromStream {
        #[ink(topic)]
        stream_id: StreamId,
        #[ink(topic)]
        recipient: AccountId,
        amount: Balance,
    }

    /// Event emitted when the [Stream] is cancelled
    #[ink(event)]
    pub struct CancelStream {
        #[ink(topic)]
        stream_id: StreamId,
        #[ink(topic)]
        sender: AccountId,
        #[ink(topic)]
        recipient: AccountId,
        sender_balance: Balance,
        recipient_balance: Balance,
    }

    use ink_storage::traits::{PackedLayout, SpreadLayout};

    /// Unique identifier for a [Stream]
    pub type StreamId = u128;

    /// A payment that takes place over a period of time
    #[derive(Debug, Encode, Decode, SpreadLayout, PackedLayout, Clone)]
    #[cfg_attr(test, derive(Eq, PartialEq))]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout))]
    pub struct Stream {
        /// the amount of money to be streamed
        pub deposit: Balance,
        /// the number of tokens allocated each second to the recipient
        pub rate_per_second: Balance,
        /// the amount left in the stream
        pub remaining_balance: Balance,
        /// the unix timestamp for when the stream starts
        pub start_time: Timestamp,
        /// the unix timestamp for when the stream stops
        pub stop_time: Timestamp,
        /// the address towards which the money is streamed
        pub recipient: AccountId,
        /// the address of the party funding the stream
        pub sender: AccountId,
        /// the ERC20 token to use as streaming currency
        pub token_address: AccountId,
        /// indicates whether the stream exists or not
        pub is_entity: bool,
    }

    impl Stream {
        /// Returns either the delta between `now` and `start_time` or between `stop_time` and
        /// `start_time`, whichever is smaller. If `now` is before `start_time`, it returns 0.
        pub fn delta_seconds(&self, now: Timestamp) -> Timestamp {
            if now <= self.start_time {
                return 0;
            }
            if now < self.stop_time {
                return core::time::Duration::from_millis(now - self.start_time).as_secs();
            }
            core::time::Duration::from_millis(self.stop_time - self.start_time).as_secs()
        }

        /// The amount that has been withdrawn so far
        pub fn amount_withdrawn(&self) -> Balance { self.deposit - self.remaining_balance }

        /// The ERC-20 token
        #[inline]
        pub fn token(&self) -> Erc20 { get_erc20(self.token_address) }

        /// Get the balance for `who` at `now`
        pub fn get_balance(&self, who: AccountId, time: Timestamp) -> Balance {
            let time_delta: Balance = self.delta_seconds(time).into();
            let recipient_balance = (time_delta * self.rate_per_second) - self.amount_withdrawn();

            // return appropriate balance
            if who == self.recipient {
                recipient_balance
            } else if who == self.sender {
                self.remaining_balance - recipient_balance
            } else {
                0
            }
        }
    }

    impl Erc1620 {
        /// Creates a new ERC-1620 contract instance
        #[allow(clippy::new_without_default)]
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                streams_by_id: Default::default(),
                next_stream_id: 1.into(),
                stream_ids_by_account: Default::default(),
            }
        }

        /// Creates a new stream funded by the caller and paid towards `recipient`.
        #[ink(message)]
        pub fn create_stream(
            &mut self,
            recipient: AccountId,
            deposit: Balance,
            token_address: AccountId,
            start_time: Timestamp,
            stop_time: Timestamp,
        ) -> Result<StreamId> {
            let caller = self.env().caller();

            // validate recipient
            if recipient == ZERO_ACCOUNT || recipient == caller || recipient == self.env().account_id() {
                return Err(Error::InvalidRecipient);
            }

            // validate time
            if stop_time < start_time {
                return Err(Error::InvalidStopTime);
            }
            let now = self.env().block_timestamp();
            if start_time < now {
                return Err(Error::InvalidStartTime);
            }

            // validate deposit
            let duration = core::time::Duration::from_millis(stop_time - start_time).as_secs().into();
            if deposit < duration {
                return Err(Error::DepositSmallerThanTimeDelta);
            }
            if deposit % duration != 0 {
                return Err(Error::DepositNotMultipleOfZero);
            }

            // transfer tokens to contract
            #[cfg(not(test))]
            get_erc20(token_address).transfer_from(caller, self.env().account_id(), deposit)?;

            // write storage
            let stream_id = self.increment_next_stream_id();
            // let token_account_id = token_address.to_account_id();
            self.streams_by_id.insert(stream_id, Stream {
                deposit,
                rate_per_second: deposit / duration,
                remaining_balance: deposit,
                start_time,
                stop_time,
                recipient,
                sender: caller,
                token_address,
                is_entity: true,
            });

            self.stream_ids_by_account
                .entry(recipient)
                // This is only valid if produced stream ids are guaranteed to be incrementing.
                .and_modify(|v| v.push(stream_id))
                .or_insert(vec![stream_id]);

            // emit event
            self.env().emit_event(CreateStream {
                stream_id,
                sender: caller,
                recipient,
                deposit,
                token_address,
                start_time,
                stop_time,
            });

            Ok(stream_id)
        }

        /// Withdraws from the contract to the recipient's account.
        #[ink(message)]
        pub fn withdraw_from_stream(&mut self, stream_id: StreamId, amount: Balance) -> Result<bool> {
            let caller = self.env().caller();

            // validate amount
            if amount == 0 {
                return Err(Error::AmountCannotBeZero);
            }

            let (remaining_balance, recipient) = {
                let now = self.env().block_timestamp();
                let stream = self.streams_by_id.get_mut(&stream_id).ok_or(Error::StreamNotFound)?;

                if caller != stream.recipient && caller != stream.sender {
                    return Err(Error::OnlyCallableBySenderOrRecipient);
                }

                // validate balance of recipient
                if stream.get_balance(stream.recipient, now) < amount {
                    return Err(Error::InsufficientBalance);
                }

                // remove the amount from balance in storage
                stream.remaining_balance -= amount;

                // transfer the tokens
                #[cfg(not(test))]
                stream.token().transfer(stream.recipient, amount)?;

                (stream.remaining_balance, stream.recipient)
            };

            // remove the stream if it's empty
            if remaining_balance == 0 {
                self.streams_by_id.take(&stream_id);
            }

            // emit event
            self.env().emit_event(WithdrawFromStream { stream_id, recipient, amount });

            Ok(true)
        }

        /// Cancels the stream and transfers the tokens back on a pro ratea basis.
        #[ink(message)]
        pub fn cancel_stream(&mut self, stream_id: StreamId) -> Result<bool> {
            let stream = self.streams_by_id.take(&stream_id).ok_or(Error::StreamNotFound)?;
            let now = self.env().block_timestamp();
            let sender_balance = stream.get_balance(stream.sender, now);
            let recipient_balance = stream.get_balance(stream.recipient, now);

            // transfer tokens
            if recipient_balance > 0 {
                #[cfg(not(test))]
                stream.token().transfer(stream.recipient, recipient_balance)?;
            }
            if sender_balance > 0 {
                #[cfg(not(test))]
                stream.token().transfer(stream.sender, sender_balance)?;
            }

            // The stream_ids should always be present, as the streams_by_id inserted stream is only
            // added in create_stream, which also always inserts streams_by_accounts. If this errors
            // the contract is in a bad state and practically unrecoverable.
            let streams_ids = self.stream_ids_by_account.get_mut(&stream.recipient).unwrap();

            // the stream id is guaranteed to be present, thus this unwrap cannot fail.
            let index = streams_ids.binary_search(&stream_id).unwrap();
            streams_ids.remove(index);

            self.env().emit_event(CancelStream {
                stream_id,
                sender: stream.sender,
                recipient: stream.recipient,
                sender_balance,
                recipient_balance,
            });
            Ok(true)
        }

        /// Withdraws the total available value from all streams where the caller is the recipient.
        #[ink(message)]
        pub fn withdraw_from_all_streams(&mut self) -> Result<Balance> {
            let caller = self.env().caller();
            let now = self.env().block_timestamp();
            let streams = self.stream_ids_by_account.get(&caller).ok_or(Error::StreamsNotFound)?.clone();
            let mut total: Balance = 0;

            for id in streams {
                // stream is always present if streams_by_accounts has the id, thus this unwrap does
                // not fail.
                let stream = self.streams_by_id.get(&id).unwrap();
                let amount = stream.get_balance(caller, now);
                total += amount;
                self.withdraw_from_stream(id, amount)?;
            }
            Ok(total)
        }

        /// Withdraws the total available value from all streams where the caller is the recipient.
        #[ink(message)]
        pub fn stream_ids(&self, account: AccountId) -> Option<Vec<StreamId>> {
            self.stream_ids_by_account.get(&account).cloned()
        }

        /// Returns the stream with id `stream_id`
        #[ink(message)]
        pub fn get_stream(&self, stream_id: StreamId) -> Option<Stream> { self.streams_by_id.get(&stream_id).cloned() }

        /// Returns the real-time balance of the account with address `who`.
        #[ink(message)]
        pub fn balance_of(&self, stream_id: StreamId, who: AccountId) -> Result<Balance> {
            Ok(self
                .streams_by_id
                .get(&stream_id)
                .ok_or(Error::StreamNotFound)?
                .get_balance(who, self.env().block_timestamp()))
        }
    }

    #[ink(impl)]
    impl Erc1620 {
        /// Get the next stream id and increment it
        fn increment_next_stream_id(&mut self) -> StreamId {
            let stream_id = *self.next_stream_id;
            *self.next_stream_id += 1;
            stream_id
        }

        /// Get the current time
        #[cfg(test)]
        fn now() -> Timestamp { Self::env().block_timestamp() }
    }

    /// Gets an ERC-20 token from an account id
    fn get_erc20(account_id: AccountId) -> Erc20 { FromAccountId::from_account_id(account_id) }

    #[cfg(test)]
    mod tests {
        use super::*;
        use contract_utils::test_utils;

        /// Validate creating and withdrawing from a stream
        #[ink::test]
        fn test_create_and_withdraw() {
            let accounts = contract_utils::test_utils::default_accounts();
            let mut instance = Erc1620::new();
            let start_time = Erc1620::now();

            // create a stream and validate it
            let stream_id =
                instance.create_stream(accounts.bob, 10_000, ZERO_ACCOUNT, start_time, start_time + 10_000).unwrap();
            let stream = instance.get_stream(stream_id).unwrap();
            assert_eq!(stream, Stream {
                deposit: 10_000,
                rate_per_second: 1_000,
                remaining_balance: 10_000,
                start_time,
                stop_time: start_time + 10_000,
                recipient: accounts.bob,
                sender: accounts.alice,
                token_address: ZERO_ACCOUNT,
                is_entity: true
            });

            // check balnaces at start
            assert_eq!(stream.get_balance(accounts.alice, start_time), 10_000);
            assert_eq!(stream.get_balance(accounts.bob, start_time), 0);

            // check balances halfway
            let time = start_time + 5000;
            assert_eq!(stream.get_balance(accounts.alice, time), 5_000);
            assert_eq!(stream.get_balance(accounts.bob, time), 5_000);

            // check balances at end
            let time = start_time + 10_000;
            assert_eq!(stream.get_balance(accounts.alice, time), 0);
            assert_eq!(stream.get_balance(accounts.bob, time), 10_000);

            // should fail because no time has passed
            instance.withdraw_from_stream(stream_id, 1_000).unwrap_err();

            test_utils::advance_time(5000);
            instance.withdraw_from_stream(stream_id, 1_000).unwrap();

            // bob can also withdraw
            test_utils::set_caller(accounts.bob);
            instance.withdraw_from_stream(stream_id, 1_000).unwrap();

            // check the stream
            let stream = instance.get_stream(stream_id).unwrap();
            assert_eq!(stream.remaining_balance, 8_000);

            // 3000 remaining because half time has passed and some has been taken out
            assert_eq!(stream.get_balance(accounts.bob, Erc1620::now()), 3_000);

            // advance the rest of the time
            test_utils::advance_time(7_000);

            // try to withdraw too much
            instance.withdraw_from_stream(stream_id, 8_001).unwrap_err();

            // withdraw the rest of the tokens and make sure the stream is deleted
            instance.withdraw_from_stream(stream_id, 8_000).unwrap();
            assert!(instance.get_stream(stream_id).is_none());
        }

        /// Validate cancelling a stream
        #[ink::test]
        fn test_cancel() {
            let accounts = contract_utils::test_utils::default_accounts();
            let mut instance = Erc1620::new();
            let start_time = Erc1620::now();

            // create a stream and validate it
            let stream_id =
                instance.create_stream(accounts.bob, 10_000, ZERO_ACCOUNT, start_time, start_time + 10_000).unwrap();
            instance.get_stream(stream_id).unwrap();

            // cancel the stream and make sure it doesn't exist
            instance.cancel_stream(stream_id).unwrap();
            assert!(instance.get_stream(stream_id).is_none());
        }

        #[ink::test]
        fn test_multiple_streams_are_updated() {
            let accounts = contract_utils::test_utils::default_accounts();
            let mut instance = Erc1620::new();
            let start_time = Erc1620::now();
            let total = 100;

            for i in 1..total {
                let stream_id =
                    instance.create_stream(accounts.bob, 100, ZERO_ACCOUNT, start_time, start_time + 10_000).unwrap();
                // we expect the streams to increment one by one.
                assert_eq!(i, stream_id)
            }

            for i in total..1 {
                instance.cancel_stream(i).unwrap();
                assert_eq!(instance.stream_ids(accounts.bob).unwrap().len(), i as usize)
            }
        }

        #[ink::test]
        fn test_withdraw_from_all() {
            let accounts = contract_utils::test_utils::default_accounts();
            let mut instance = Erc1620::new();
            let start_time = Erc1620::now();
            let total = 100;

            for i in 1..=total {
                let stream_id = instance
                    .create_stream(accounts.bob, 10_000, ZERO_ACCOUNT, start_time, start_time + 10_000)
                    .unwrap();
                // we expect the streams to increment one by one.
                assert_eq!(i, stream_id)
            }

            contract_utils::test_utils::set_caller(accounts.bob);
            contract_utils::test_utils::advance_time(5_000);
            let balance = instance.withdraw_from_all_streams().unwrap();
            assert_eq!(balance, total * 5_000)
        }
    }
}
