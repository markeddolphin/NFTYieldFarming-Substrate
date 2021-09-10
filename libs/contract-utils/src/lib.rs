#![cfg_attr(not(feature = "std"), no_std)]

pub mod constants;
mod log;
pub mod math;
pub mod test_utils;
pub mod time;
mod token;

pub use log::*;
pub use token::*;

#[cfg(feature = "decimal")]
pub use rust_decimal;

use env_exports::*;
use ink_env::Environment;
use ink_lang::EnvAccess;

/// These exports are added automatically by ink, but they are not detected by the IDE, so reexport them manually
pub mod env_exports {
    pub type Balance = <ink_env::DefaultEnvironment as ink_env::Environment>::Balance;
    pub type Hash = <ink_env::DefaultEnvironment as ink_env::Environment>::Hash;
    pub type Timestamp = <ink_env::DefaultEnvironment as ink_env::Environment>::Timestamp;
    pub use ink_env::AccountId;
}

pub const ZERO_ACCOUNT: ink_env::AccountId = unsafe { core::mem::transmute([0_u8; 32]) };

/// Extensions for AccountId
pub trait AccountIdExt {
    /// True if the value is 0
    fn is_zero(&self) -> bool;
    /// Convert AccountId to bytes
    fn into_bytes(self) -> [u8; 32];
}

impl AccountIdExt for AccountId {
    fn is_zero(&self) -> bool { self == &ZERO_ACCOUNT }

    // This will fail if length of bytes is changed
    fn into_bytes(self) -> [u8; 32] { unsafe { core::mem::transmute(self) } }
}

/// Extensions for `Hash`
pub trait HashExt {
    fn to_account_id(&self) -> AccountId;
}

impl HashExt for Hash {
    fn to_account_id(&self) -> AccountId {
        use core::convert::TryFrom;

        AccountId::try_from(self.as_ref()).expect("Hash and AccountId must be same length")
    }
}

/// Gets the reward account for an account
pub fn get_reward_account_id<T: Environment>(environment: EnvAccess<T>, account_id: AccountId) -> AccountId {
    let mut bytes = [0_u8; 46];
    bytes[0..14].copy_from_slice(b"reward_address");
    bytes[14..].copy_from_slice(&account_id.into_bytes());
    environment.hash_bytes::<ink_env::hash::Keccak256>(&bytes).into()
}

/// Assert that the caller is the owner (requires an `owner` field of type `AccountId`)
#[macro_export]
macro_rules! assert_caller_is_owner {
    ($self:ident) => {
        assert_eq!($self.env().caller(), $self.owner);
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use scale::{Decode, Encode};

    /// Converting AccountId to bytes works correctly
    #[test]
    fn test_account_id_to_bytes() {
        let raw = [2_u8; 32];
        assert_eq!(AccountId::from(raw).into_bytes(), raw);
    }

    /// Convert a Hash to an AccountId
    #[test]
    fn hash_to_account_id() {
        let raw = [2_u8; 32];
        assert_eq!(Hash::from(raw).to_account_id(), raw.into());
    }

    /// Make sure encoding decoding a Hash works
    #[test]
    fn test_hash_encode_decode() {
        let hash = Hash::from([7_u8; 32]);
        assert_eq!(Hash::decode(&mut &hash.encode()[..]).unwrap(), hash);
    }
}
