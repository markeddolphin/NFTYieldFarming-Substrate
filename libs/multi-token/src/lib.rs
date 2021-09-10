#![cfg_attr(not(feature = "std"), no_std)]

//! This crate unifies the API across ERC-20, ERC-721, and ERC-1155.

mod unique;

pub use unique::{UniqueMultiToken, UniqueMultiTokenInfo};

use contract_utils::{env_exports::*, TokenStandard};
use erc1155::Erc1155;
use erc20::Erc20;
use erc721::Erc721;
use ink_env::call::FromAccountId;
use ink_prelude::vec::Vec;
use ink_storage::traits::{PackedLayout, SpreadLayout, StorageLayout};
use scale::{Decode, Encode};

/// TokenId type used by ERC-721 and ERC-1155
pub type TokenId = u64;

/// The error type for this crate
#[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, err_derive::Error)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    /// Tried to convert to an unsupported token standard
    #[error(display = "Tried to convert to an unsupported token standard")]
    InvalidTokenStandard,
    /// A balance is required, but it was not provided
    #[error(display = "A balance is required, but it was not provided")]
    BalanceRequired,
    /// A token ID is required, but it was not provided
    #[error(display = "A token ID is required, but it was not provided")]
    TokenIdRequired,
    /// Metadata is required, but it was not provided
    #[error(display = "Metadata is required, but it was not provided")]
    MetadataRequired,
    /// Token account not found
    #[error(display = "token account not found for name: {}", _0)]
    TokenNotFound(ink_prelude::string::String),
    /// ERC-20 error
    #[error(display = "ERC-20 error: {}", _0)]
    Erc20(#[source] erc20::Error),
    /// ERC-721 error
    #[error(display = "ERC-721 error: {}", _0)]
    Erc721(#[source] erc721::Error),
    /// ERC-1155 error
    #[error(display = "ERC-1155 error: {}", _0)]
    Erc1155(#[source] erc1155::Error),
}

/// The Result type for this crate
pub type Result<T> = core::result::Result<T, Error>;

/// A token that can be one of multiple standards
#[derive(Debug, Encode, Decode, SpreadLayout, PackedLayout, Clone, Copy)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo, ink_storage::traits::StorageLayout))]
pub struct MultiToken {
    /// The `AccountId` of the deployed token contract
    pub account_id: AccountId,
    /// The standard the token adheres to
    pub standard: TokenStandard,
}

impl MultiToken {
    /// Create a new instance
    pub fn new(account_id: AccountId, standard: TokenStandard) -> Self { Self { account_id, standard } }

    /// Transfer `amount` tokens with id `token_id` to `to`. `token_id` and `amount` are optional in some
    /// for some standards, but will cause an error if they're required.
    pub fn transfer(
        &mut self,
        to: AccountId,
        token_id: impl Into<Option<u64>>,
        amount: impl Into<Option<Balance>>,
    ) -> Result<()> {
        match self.standard {
            TokenStandard::Erc20 => {
                self.as_erc20_unchecked().transfer(to, amount.into().ok_or(Error::BalanceRequired)?)?
            }
            TokenStandard::Erc721 => {
                self.as_erc721_unchecked().transfer(to, token_id.into().ok_or(Error::TokenIdRequired)?)?
            }
            TokenStandard::Erc1155 => self.as_erc1155_unchecked().transfer(
                to,
                token_id.into().ok_or(Error::TokenIdRequired)?,
                amount.into().ok_or(Error::BalanceRequired)?,
            )?,
        }
        Ok(())
    }

    /// Transfer `amount` tokens with id `token_id` from `from` to `to`. `token_id` and `amount` are optional in some
    /// for some standards, but will cause an error if they're required.
    pub fn transfer_from(
        &mut self,
        from: AccountId,
        to: AccountId,
        token_id: impl Into<Option<TokenId>>,
        amount: impl Into<Option<Balance>>,
    ) -> Result<()> {
        match self.standard {
            TokenStandard::Erc20 => {
                self.as_erc20_unchecked().transfer_from(from, to, amount.into().ok_or(Error::BalanceRequired)?)?
            }
            TokenStandard::Erc721 => {
                self.as_erc721_unchecked().transfer_from(from, to, token_id.into().ok_or(Error::TokenIdRequired)?)?
            }
            TokenStandard::Erc1155 => self.as_erc1155_unchecked().transfer_from(
                from,
                to,
                token_id.into().ok_or(Error::TokenIdRequired)?,
                amount.into().ok_or(Error::BalanceRequired)?,
            )?,
        }
        Ok(())
    }

    /// Allows `spender` to withdraw from the caller's account multiple times, up to the `amount`.
    /// If this function is called again it overwrites the current allowance with `value`.
    pub fn approve(
        &mut self,
        spender: AccountId,
        token_id: impl Into<Option<TokenId>>,
        amount: impl Into<Option<Balance>>,
    ) -> Result<()> {
        match self.standard {
            TokenStandard::Erc20 => {
                self.as_erc20_unchecked().approve(spender, amount.into().ok_or(Error::BalanceRequired)?)?
            }
            TokenStandard::Erc721 => {
                self.as_erc721_unchecked().approve(spender, token_id.into().ok_or(Error::TokenIdRequired)?)?
            }
            // TODO: it seems like amount should be used with Erc1155
            TokenStandard::Erc1155 => {
                self.as_erc1155_unchecked().approve(spender, token_id.into().ok_or(Error::TokenIdRequired)?)?
            }
        }
        Ok(())
    }

    /// Returns the amount which `spender` is allowed to withdraw from `owner`.
    pub fn allowance(&mut self, owner: AccountId, spender: AccountId) -> Option<Balance> {
        match self.standard {
            TokenStandard::Erc20 => Some(self.as_erc20_unchecked().allowance(owner, spender)),
            TokenStandard::Erc721 => {
                // TODO: implement allowance for erc721
                unimplemented!("allowance is not implemented for erc721")
            }
            TokenStandard::Erc1155 => {
                // TODO: implement allowance for erc1155
                unimplemented!("allowance is not implemented for erc1155")
            }
        }
    }

    /// Mint `amount` tokens to `recipient` with `metadata`
    pub fn mint(
        &mut self,
        recipient: AccountId,
        amount: impl Into<Option<Balance>>,
        metadata: impl Into<Option<Vec<u8>>>,
    ) -> Result<()> {
        match self.standard {
            TokenStandard::Erc20 => {
                self.as_erc20_unchecked().mint(recipient, amount.into().ok_or(Error::BalanceRequired)?)?
            }
            TokenStandard::Erc721 => {
                self.as_erc721_unchecked().mint_with_metadata(recipient, metadata.into().unwrap_or_default())?;
            }
            TokenStandard::Erc1155 => self.as_erc1155_unchecked().mint(
                recipient,
                amount.into().ok_or(Error::BalanceRequired)?,
                metadata.into().ok_or(Error::MetadataRequired)?,
            )?,
        }
        Ok(())
    }

    /// Burn `amount` tokens with id `token_id`
    pub fn burn(&mut self, token_id: impl Into<Option<TokenId>>, amount: impl Into<Option<Balance>>) -> Result<()> {
        match self.standard {
            TokenStandard::Erc20 => self.as_erc20_unchecked().burn(amount.into().ok_or(Error::BalanceRequired)?)?,
            TokenStandard::Erc721 => self.as_erc721_unchecked().burn(token_id.into().ok_or(Error::TokenIdRequired)?)?,
            TokenStandard::Erc1155 => self
                .as_erc1155_unchecked()
                .burn(token_id.into().ok_or(Error::TokenIdRequired)?, amount.into().ok_or(Error::BalanceRequired)?)?,
        }
        Ok(())
    }

    /// Burn `amount` tokens from `account` with id `token_id`
    pub fn burn_from(
        &mut self,
        account: AccountId,
        token_id: impl Into<Option<TokenId>>,
        amount: impl Into<Option<Balance>>,
    ) -> Result<()> {
        match self.standard {
            TokenStandard::Erc20 => {
                self.as_erc20_unchecked().burn_from(account, amount.into().ok_or(Error::BalanceRequired)?)?
            }
            TokenStandard::Erc721 => {
                self.as_erc721_unchecked().burn_from(account, token_id.into().ok_or(Error::TokenIdRequired)?)?
            }
            TokenStandard::Erc1155 => self.as_erc1155_unchecked().burn_from(
                account,
                token_id.into().ok_or(Error::TokenIdRequired)?,
                amount.into().ok_or(Error::BalanceRequired)?,
            )?,
        }
        Ok(())
    }

    /// Returns the balance of `account`
    pub fn balance_of(&mut self, account: AccountId) -> Balance {
        match self.standard {
            TokenStandard::Erc20 => self.as_erc20_unchecked().balance_of(account),
            TokenStandard::Erc721 => {
                // TODO: implement allowance for erc721
                unimplemented!("allowance is not implemented for erc721")
            }
            TokenStandard::Erc1155 => {
                // TODO: implement allowance for erc1155
                unimplemented!("allowance is not implemented for erc1155")
            }
        }
    }

    /// Safe convert to Erc20
    pub fn as_erc20(&self) -> Result<Erc20> {
        if self.standard != TokenStandard::Erc20 {
            return Err(Error::InvalidTokenStandard);
        }
        Ok(self.as_erc20_unchecked())
    }

    /// Convert to Erc20 without checking the token standard
    pub fn as_erc20_unchecked(&self) -> Erc20 { FromAccountId::from_account_id(self.account_id) }

    /// Safe convert to Erc721
    pub fn as_erc721(&self) -> Result<Erc721> {
        if self.standard != TokenStandard::Erc721 {
            return Err(Error::InvalidTokenStandard);
        }
        Ok(FromAccountId::from_account_id(self.account_id))
    }

    /// Convert to Erc721 without checking the token standard
    pub fn as_erc721_unchecked(&self) -> Erc721 { FromAccountId::from_account_id(self.account_id) }

    /// Safely Convert to Erc1155
    pub fn as_erc1155(&self) -> Result<Erc1155> {
        if self.standard != TokenStandard::Erc1155 {
            return Err(Error::InvalidTokenStandard);
        }
        Ok(FromAccountId::from_account_id(self.account_id))
    }

    /// Convert to Erc1155 without checking the token standard
    pub fn as_erc1155_unchecked(&self) -> Erc1155 { FromAccountId::from_account_id(self.account_id) }
}

#[cfg(feature = "token-accounts")]
impl From<token_accounts::Token> for MultiToken {
    fn from(token: token_accounts::Token) -> Self { Self { account_id: token.account_id, standard: token.standard } }
}
