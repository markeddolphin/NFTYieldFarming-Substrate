#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unused_must_use)]

pub use contract::TokenAccounts;

use contract_utils::TokenStandard;
use ink_env::AccountId;
use ink_lang as ink;
use ink_prelude::{string::String, vec::Vec};
use ink_storage::traits::{PackedLayout, SpreadLayout};
use scale::{Decode, Encode};

/// Error types
#[derive(Debug, Copy, Clone, PartialEq, Eq, Encode, Decode, err_derive::Error)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    /// Only the owner may perform this operation
    #[error(display = "Only the owner may perform this operation")]
    OnlyOwnerAllowed,
}

/// The result type for this contract
pub type Result<T> = core::result::Result<T, Error>;

/// Contains data for a token
#[derive(Debug, Encode, Decode, SpreadLayout, PackedLayout, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout))]
pub struct Token {
    /// The token's account id
    pub account_id: AccountId,
    /// The token's standard
    pub standard: TokenStandard,
}

impl Token {
    /// Create a new instance
    pub fn new(account_id: AccountId, standard: TokenStandard) -> Self { Self { account_id, standard } }
}

/// Information about a token
#[derive(Debug, Encode, Decode, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub struct TokenInfo {
    /// The token's symbol (the key)
    pub symbol: String,
    /// The token's account
    pub account_id: AccountId,
    /// The token's standard
    pub standard: TokenStandard,
}

#[ink::contract]
mod contract {
    use super::*;

    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_storage::{collections::HashMap, lazy::Lazy};

    /// Event emitted when `set_account` is successful
    #[ink(event)]
    pub struct SetToken {
        #[ink(topic)]
        symbol: String,
        #[ink(topic)]
        account_id: AccountId,
        #[ink(topic)]
        standard: TokenStandard,
    }

    /// Event emitted when `set_account` is successful
    #[ink(event)]
    pub struct RemovedToken {
        #[ink(topic)]
        symbol: String,
    }

    /// Contains info for tokens
    #[ink(storage)]
    pub struct TokenAccounts {
        /// `AccountId` by token symbol
        tokens_by_symbol: HashMap<String, Token>,
        /// The owner of the contract
        owner: Lazy<AccountId>,
    }

    impl TokenAccounts {
        /// Creates a new instance
        #[ink(constructor)]
        #[allow(clippy::new_without_default)]
        pub fn new() -> Self { Self { tokens_by_symbol: Default::default(), owner: Lazy::new(Self::env().caller()) } }

        /// Insert a token
        #[ink(message)]
        pub fn set_token(&mut self, symbol: String, account_id: AccountId, standard: TokenStandard) -> Result<()> {
            if self.env().caller() != *self.owner {
                return Err(Error::OnlyOwnerAllowed);
            }
            self.tokens_by_symbol.insert(symbol.clone(), Token { account_id, standard });
            self.env().emit_event(SetToken { symbol, account_id, standard });
            Ok(())
        }

        /// Remove a token
        #[ink(message)]
        pub fn remove_token(&mut self, symbol: String) -> Result<()> {
            if self.env().caller() != *self.owner {
                return Err(Error::OnlyOwnerAllowed);
            }
            self.tokens_by_symbol.take(&symbol);
            self.env().emit_event(RemovedToken { symbol });
            Ok(())
        }

        /// Returns the `Token` for the given `symbol`
        #[ink(message)]
        pub fn get_token(&self, symbol: String) -> Option<Token> { self.tokens_by_symbol.get(&symbol).copied() }

        /// Returns all of the tokens
        #[ink(message)]
        pub fn get_all_tokens(&self) -> Vec<TokenInfo> {
            self.tokens_by_symbol
                .iter()
                .map(|(symbol, x)| TokenInfo { symbol: symbol.clone(), account_id: x.account_id, standard: x.standard })
                .collect()
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use contract_utils::test_utils;
        use ink_prelude::vec;

        #[ink::test]
        fn test_set_get_account() {
            let mut tokens = TokenAccounts::new();
            let symbol = String::from("USDT");
            let accounts = test_utils::default_accounts();

            // add token
            tokens.set_token(symbol.clone(), accounts.bob, TokenStandard::Erc20).unwrap();
            assert_eq!(test_utils::recorded_event_count(), 1);
            assert_eq!(tokens.get_token(symbol.clone()).unwrap().account_id, accounts.bob);

            // add another token and get both
            tokens.set_token("ETH".into(), accounts.charlie, TokenStandard::Erc20).unwrap();
            assert_eq!(tokens.get_all_tokens(), vec![
                TokenInfo { symbol: "USDT".into(), account_id: accounts.bob, standard: TokenStandard::Erc20 },
                TokenInfo { symbol: "ETH".into(), account_id: accounts.charlie, standard: TokenStandard::Erc20 }
            ]);

            // remove the token
            tokens.remove_token(symbol.clone()).unwrap();
            assert_eq!(test_utils::recorded_event_count(), 3);
            assert!(tokens.get_token(symbol.clone()).is_none());

            // calling by non-owner should fail
            test_utils::set_caller(accounts.charlie);
            tokens.set_token(symbol.clone(), accounts.bob, TokenStandard::Erc20).unwrap_err();
            tokens.remove_token(symbol).unwrap_err();
        }
    }
}
