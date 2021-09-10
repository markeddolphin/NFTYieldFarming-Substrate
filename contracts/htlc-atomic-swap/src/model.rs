use contract_utils::env_exports::*;
use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
use multi_token::{UniqueMultiToken, UniqueMultiTokenInfo};
use scale::{Decode, Encode};

pub mod storage {
    use super::*;

    /// A hashed time-locked contract
    #[derive(Debug, Encode, Decode, SpreadLayout, PackedLayout, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo, StorageLayout))]
    pub struct HTLContract {
        /// Hash of the secret of the HTLC
        pub secret_hash: Hash,
        /// Address of the From (generator of the proposal)
        pub from: AccountId,
        /// Address of the receiver of the funds
        pub to: AccountId,
        /// Token for the transfer
        pub token: UniqueMultiToken,
        /// Amount of the transaction
        pub amount: Balance,
        /// Time that the contract expires
        pub time_lock: u64,
        /// If the contract is locked
        pub locked: bool,
    }
}

pub mod input {
    use super::*;

    /// Used to claim funds
    #[derive(Debug, Encode, Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct ClaimRequest {
        /// The hash of the contract (the key)
        pub contract_hash: Hash,
        /// Secret used to access the funds
        pub secret: Hash,
    }

    /// Used to request funds
    #[derive(Debug, Encode, Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct RefundRequest {
        /// The contract hash (key)
        pub contract_hash: Hash,
        /// The hash of the secret
        pub secret_hash: Hash,
    }

    /// Used when initializing an HTLC
    #[derive(Debug, Encode, Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct Proposal {
        /// Address of the receiver of the funds
        pub to: AccountId,
        /// Symbol of the token for transfer
        pub token: UniqueMultiTokenInfo,
        /// Amount of the transaction
        pub amount: Balance,
        /// Timestamp that the contract expires
        pub time_lock: u64,
        /// Hash of the secret of the HTLC
        pub secret_hash: Hash,
    }
}

pub mod output {
    use super::*;

    #[derive(Debug, Encode, Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct HTLContractOutput {
        pub secret_hash: Hash,
        pub from: AccountId,
        pub to: AccountId,
        pub token: UniqueMultiTokenInfo,
        pub amount: Balance,
        pub time_lock: u64,
        pub locked: bool,
    }
}

pub mod event_output {
    use super::*;

    /// A proposal was opened
    /// This is only sent it from or to is zero
    #[derive(Debug, Encode, Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct InitHTLCEventOutput {
        /// From is an address of a sender (generator of the proposal)
        pub from: AccountId,
        /// To is an address of the receiver of the funds
        pub to: AccountId,
        // TODO: `token`` is temporarily removed due to issues with redspot
        // /// Token for the transfer
        // pub token: UniqueMultiTokenInfo,
        /// Amount of the transaction
        pub amount: Balance,
        /// TimeLock until that the contract expires
        pub time_lock: u64,
        /// SecretHash of the HTLC that will be later checked against a secret
        pub secret_hash: Hash,
    }

    /// Sent when a contract is created
    #[derive(Debug, Encode, Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct CreatedContractOutput {
        /// The hash of the contract that was created
        pub contract_hash: Hash,
    }

    /// ClaimFundsEvent is a payload of an event that is emitted when receiver claims his funds
    #[derive(Debug, Encode, Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct ClaimFundsEventOutput {
        /// Address of the receiver (claimer)
        pub address: AccountId,
        /// ContractHash of a HLTC contract
        pub contract_hash: Hash,
        /// Secret of the HTLC
        pub secret: Hash,
    }

    /// RefundFundsEvent is a payload of an event that is emitted when funds are being transferred from HTLC address
    /// to a receiver
    #[derive(Debug, Encode, Decode, Clone)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct RefundFundsEventOutput {
        /// Address of the receiver (claimer)
        pub address: AccountId,
        /// ContractHash of a HLTC contract
        pub contract_hash: Hash,
        /// Secret of the HTLC
        pub secret: Hash,
    }
}
