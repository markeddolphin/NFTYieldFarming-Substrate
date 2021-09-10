#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unused_must_use)]

mod model;

use contract_utils::env_exports::*;
use scale::{Decode, Encode};

/// Error types
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, err_derive::Error)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    /// HTLC can not be a swap-in and swap-out simultaneously
    #[error(display = "HTLC can not be a swap-in and swap-out simultaneously")]
    InvalidAccounts,
    /// Contract is expired
    #[error(display = "Contract is expired")]
    ContractExpired,
    /// Contract not found
    #[error(display = "Contract not found for hash: {:?}", _0)]
    ContractNotFound(Hash),
    /// Escrow account not found
    #[error(display = "The escrow account was not found")]
    EscrowAccountNotFound,
    /// No funds locked for the HTLC
    #[error(display = "No funds locked for the HTLC: {:?}", _0)]
    ContractNotLocked(Hash),
    /// Wrong address provided to claim funds
    #[error(display = "The address {:?} has no rights to claim the funds", _0)]
    InvalidClaimer(AccountId),
    /// Incorrect secret provided
    #[error(display = "The secret provided by {:?} is incorrect", _0)]
    IncorrectSecret(AccountId),
    /// Only the owner pay perform this operation
    #[error(display = "Only the owner pay perform this operation")]
    RequiresOwner,
    /// An ERC-20 error occurred
    #[error(display = "An Erc20 error occurred: {}", _0)]
    Erc20(#[source] multi_token::Error),
}

/// The ERC-20 result type.
pub type Result<T> = core::result::Result<T, Error>;

/// Enables minting of coins
#[ink_lang::contract]
pub mod contract {
    use super::*;

    // #[cfg(not(feature = "ink-as-dependency"))]
    use crate::model::{event_output::*, input::*, storage::*};


    #[ink(storage)]
    pub struct HashTimeLockedContracts {
        /// HTLC contracts stored by ID (a nonce)
        contracts_by_hash: ink_storage::collections::HashMap<Hash, HTLContract>,
        /// The owner of the contract
        owner: AccountId,
        /// A nonce that is used to generate the contract hash
        nonce: ink_storage::lazy::Lazy<u128>,
    }

    // ======== Events

    #[ink(event)]
    #[derive(derive_new::new)]
    pub struct InitHTLCEvent {
        /// Ouput of the event
        pub output: InitHTLCEventOutput,
    }

    /// Sent when a contract is created
    #[ink(event)]
    #[derive(derive_new::new)]
    pub struct CreatedContract {
        /// Ouput of the event
        pub output: CreatedContractOutput,
    }

    /// ClaimFundsEvent is a payload of an event that is emitted when receiver claims his funds
    #[ink(event)]
    #[derive(derive_new::new)]
    pub struct ClaimFundsEvent {
        /// Ouput of the event
        pub output: ClaimFundsEventOutput,
    }

    /// RefundFundsEvent is a payload of an event that is emitted when funds are being transferred from HTLC address
    /// to a receiver
    #[ink(event)]
    #[derive(derive_new::new)]
    pub struct RefundFundsEvent {
        /// Ouput of the event
        pub output: RefundFundsEventOutput,
    }

    impl HashTimeLockedContracts {
        /// Creates a new ERC-20 contract with the specified initial supply.
        #[allow(clippy::new_without_default)]
        #[ink(constructor)]
        pub fn new() -> Self {
            Self { contracts_by_hash: Default::default(), owner: Self::env().caller(), nonce: Default::default() }
        }

        /// Generates a proposal for a new Hash-Time Locked Contract. Returns the unique id generated for the contract.
        ///
        /// * `from` - Address of the From (generator of the proposal)
        /// * `to` - Address of the receiver of the funds
        /// * `token` - Token for the transfer
        /// * `amount` - Amount of the transaction
        /// * `time_lock` - Time until that the contract expires
        /// * `secret_hash` - Hash of the secret of the HTLC
        #[ink(message)]
        pub fn initialise_htlc(&mut self, proposal: Proposal) -> Result<()> {
            use contract_utils::AccountIdExt;

            let mut token = proposal.token.into();
            let caller = self.env().caller();

            // Generate Contract and its hash
            let mut contract = HTLContract {
                secret_hash: proposal.secret_hash,
                from: caller,
                to: proposal.to,
                // escrow_address: Default::default(),
                token,
                amount: proposal.amount,
                time_lock: proposal.time_lock,
                locked: false,
            };

            let contract_hash = {
                let mut subject = [0_u8; 32];
                subject[0..16].copy_from_slice(&self.env().caller().into_bytes()[..16]);
                subject[16..32].copy_from_slice(&self.increment_nonce().to_le_bytes());
                self.env().random(&subject)
            };

            // Check if is expired
            let timestamp: u64 = self.env().block_timestamp();
            if contract.time_lock <= timestamp {
                // ink_env::debug_println(&ink_prelude::format!("timestamp: {}", timestamp));
                return Err(Error::ContractExpired);
            }

            // Transfer funds to contract. Mint them if it's a swap-in.
            if self.caller_is_owner() {
                token.multi_token.mint(self.env().account_id(), contract.amount, None)?;
            } else {
                token.transfer_from(caller, self.env().account_id(), contract.amount)?;
            }
            contract.locked = true;

            // Update storage
            self.contracts_by_hash.insert(contract_hash, contract);

            // Send an event if swap-in or swap-out
            if self.caller_is_owner() || self.account_is_owner(proposal.to) {
                self.env().emit_event(InitHTLCEvent::new(InitHTLCEventOutput {
                    from: caller,
                    to: proposal.to,
                    // token: token.into(),
                    amount: proposal.amount,
                    time_lock: proposal.time_lock,
                    secret_hash: proposal.secret_hash,
                }));
            }

            // send event with contract hash
            self.env().emit_event(CreatedContract::new(CreatedContractOutput { contract_hash }));

            Ok(())
        }

        /// Claim the funds if the secret key is correct
        #[ink(message)]
        pub fn claim_funds(&mut self, claim: ClaimRequest) -> Result<()> {
            // Get HTLC from state
            let contract =
                self.contracts_by_hash.get(&claim.contract_hash).ok_or(Error::ContractNotFound(claim.contract_hash))?;

            // Validate HTLC has the funds locked
            if !contract.locked {
                return Err(Error::ContractNotLocked(claim.contract_hash));
            }

            // Validate HTLC has not expired
            let timestamp = self.env().block_timestamp();
            if contract.time_lock <= timestamp {
                return Err(Error::ContractExpired);
            }

            // Verify the claim is correct
            let caller = self.env().caller();

            // Verify the claimer is the receiver of the funds
            if contract.to != caller {
                return Err(Error::InvalidClaimer(caller));
            }

            // Verify the secret is correct
            let hash = HashTimeLockedContracts::hash_secret(&claim.secret);
            if hash != contract.secret_hash.as_ref() {
                return Err(Error::IncorrectSecret(caller));
            }

            // Mint funds to the claimer. If there is no claimer, burn the tokens.
            let mut multi_token = contract.token;
            if self.caller_is_owner() {
                multi_token.burn(contract.amount)?;
            } else {
                multi_token.transfer(contract.to, contract.amount)?;
            }

            let from_or_to_is_owner = self.caller_is_owner() || self.account_is_owner(contract.to);

            // Delete HTLC contract on blockchain
            self.contracts_by_hash.take(&claim.contract_hash);

            // Send event
            if from_or_to_is_owner {
                self.env().emit_event(ClaimFundsEvent::new(ClaimFundsEventOutput {
                    address: caller,
                    contract_hash: claim.contract_hash,
                    secret: claim.secret,
                }));
            }
            Ok(())
        }

        /// Returns the funds to the sender if the time lock has expired
        #[ink(message)]
        pub fn refund_funds(&mut self, refund: RefundRequest) -> Result<()> {
            // Get HTLC from state
            let contract = self
                .contracts_by_hash
                .get(&refund.contract_hash)
                .ok_or(Error::ContractNotFound(refund.contract_hash))?;
            let caller = self.env().caller();

            // Validate HTLC has the funds locked
            if !contract.locked {
                return Err(Error::ContractNotLocked(refund.contract_hash));
            }

            // Verify the claimer is the receiver of the funds
            if contract.from != caller {
                return Err(Error::InvalidClaimer(caller));
            }

            // Verify the secret Hash is correct
            if refund.secret_hash != contract.secret_hash {
                return Err(Error::IncorrectSecret(caller));
            }

            // Refund to claimer if not swap-in. Otherwise, burn the funds
            let mut multi_token = contract.token;
            if self.caller_is_owner() {
                multi_token.burn(contract.amount)?;
            } else {
                multi_token.transfer(caller, contract.amount)?;
            }

            let to_or_from_is_owner = self.caller_is_owner() || self.account_is_owner(contract.to);

            // Delete HTL contract
            self.contracts_by_hash.take(&refund.contract_hash);

            // Send an event in case of swap-in or swap-out
            if to_or_from_is_owner {
                self.env().emit_event(RefundFundsEvent::new(RefundFundsEventOutput {
                    address: caller,
                    contract_hash: refund.contract_hash,
                    secret: refund.secret_hash,
                }));
            }

            Ok(())
        }

        /// Returns information about the HTLC given the `contract_hash`
        #[ink(message)]
        pub fn get_htlc_info(&self, contract_hash: Hash) -> Option<model::output::HTLContractOutput> {
            self.contracts_by_hash.get(&contract_hash).map(|x| model::output::HTLContractOutput {
                secret_hash: x.secret_hash,
                from: x.from,
                to: x.to,
                // escrow_address: x.escrow_address,
                token: x.token.into(),
                amount: x.amount,
                time_lock: x.time_lock,
                locked: x.locked,
                // unlocked: !x.locked,
                // rolled_back: false,
            })
        }

        /// Set the owner. May only be done by the current owner.
        #[ink(message)]
        pub fn set_owner(&mut self, owner: AccountId) -> Result<()> {
            if !self.caller_is_owner() {
                return Err(Error::RequiresOwner);
            }
            self.owner = owner;
            Ok(())
        }

        /// Returns a unique number
        fn increment_nonce(&mut self) -> u128 {
            let value = *self.nonce;
            *self.nonce += 1;
            value
        }

        /// Hashes the secret using Keccak256.
        fn hash_secret(secret: &Hash) -> [u8; 32] {
            Self::env().hash_bytes::<ink_env::hash::Keccak256>(secret.as_ref())
        }

        /// True if the caller is the owner
        fn caller_is_owner(&self) -> bool { self.owner == self.env().caller() }

        /// True if `account` is the owner`
        fn account_is_owner(&self, account: AccountId) -> bool { self.owner == account }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use hex_literal::hex;

        #[test]
        fn test_secret_hash() {
            assert_eq!(
                HashTimeLockedContracts::hash_secret(
                    &hex!("7e3231d03bb0bd1cd542c20b1ff232e08d88ffd452c576558c9415414a6127ea").into()
                ),
                hex!("4c9bf8fc46df3e252c8eaf0d450d7bf95c56f4d6284a3c89af37154dc2660a39")
            )
        }
    }
}
