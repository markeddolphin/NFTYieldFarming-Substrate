#![cfg_attr(not(feature = "std"), no_std)]
#![feature(type_ascription)]

mod errors;
mod models;

use ink_lang as ink;

#[ink::contract]
mod stablecoin {
    use crate::{
        errors::{
            GetPriceError, OracleError, OwnerError, RegisterOracleError, SubmitPriceError, UpdateOracleStateError,
        },
        models::{
            Oracle, OracleState, PriceBucket, RegisterOracleRequest, SubmitPriceRequest, Ticker,
            UpdateOracleStateRequest,
        },
    };

    use multi_token::MultiToken;
    use token_accounts::TokenAccounts;

    use crate::{
        errors::ConvertError,
        models::{ConvertRequest, TokenData, TokenSpec},
    };
    use ink_env::call::FromAccountId;
    use ink_storage::{collections::HashMap, Lazy};
    use rust_decimal::Decimal;

    /// The Stablecoin smartcontract implements a simple swap between a collateral and stablecoin
    /// based on the burning and minting of the respective coins. Centralized oracles provide the
    /// data for the conversion.
    #[ink(storage)]
    pub struct Stablecoin {
        owner: Lazy<AccountId>,
        prices: HashMap<Ticker, PriceBucket>,
        oracles: HashMap<AccountId, Oracle>,

        /// Erc20 contract account id of the stable coin. (pUSD).
        stable: Lazy<TokenSpec>,

        /// Erc20 contract account id of the collateral. (Privi).
        collateral: Lazy<TokenSpec>,
    }



    /// Emitted when an oracle submits a new price. Contains the newest price state.
    #[ink(event)]
    pub struct PriceSubmitted {
        output: PriceSubmittedOutput,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct PriceSubmittedOutput {
        pub oracle: AccountId,
        pub ticker: Ticker,
    }

    impl From<PriceSubmittedOutput> for PriceSubmitted {
        fn from(output: PriceSubmittedOutput) -> Self { Self { output } }
    }

    /// Emitted when the contract owner registers a new oracle.
    #[ink(event)]
    pub struct OracleRegistered {
        pub output: OracleRegisteredOutput,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct OracleRegisteredOutput {
        pub oracle: Oracle,
    }

    impl From<OracleRegisteredOutput> for OracleRegistered {
        fn from(output: OracleRegisteredOutput) -> Self { Self { output } }
    }

    /// Emitted when the contract owner changes the state of an oracle.
    #[ink(event)]
    pub struct OracleStateUpdated {
        pub output: OracleStateUpdatedOutput,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct OracleStateUpdatedOutput {
        pub oracle: AccountId,
        pub state: OracleState,
    }


    impl From<OracleStateUpdatedOutput> for OracleStateUpdated {
        fn from(output: OracleStateUpdatedOutput) -> Self { Self { output } }
    }

    /// Emitted whenever a conversion takes place.
    #[ink(event)]
    pub struct Conversion {
        output: ConversionOutput,
    }

    #[derive(Debug, PartialEq, Eq, scale::Encode, scale::Decode)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub struct ConversionOutput {
        pub from: Ticker,
        pub to: Ticker,
        pub caller: AccountId,
    }

    impl From<ConversionOutput> for Conversion {
        fn from(output: ConversionOutput) -> Self { Self { output } }
    }

    impl Stablecoin {
        /// Constructs the contract. Note that it uses the token-accounts contract to determine the
        /// actual assets, which can thus be swapped by changing the assets in the token-accounts
        /// contract.
        ///
        /// # Restrictions
        ///
        /// The conctract creator must ensure that the stable and collateral tickers are registered
        /// with the token-accounts contract, and that this contract has mint and burn roles for both.
        #[ink(constructor)]
        pub fn new(stable: Ticker, collateral: Ticker, token_accounts: AccountId) -> Self {
            let token_accounts: TokenAccounts = FromAccountId::from_account_id(token_accounts);
            let stable =
                TokenSpec::new(token_accounts.get_token(stable).map(MultiToken::from).unwrap().as_erc20().unwrap())
                    .unwrap();
            let collateral =
                TokenSpec::new(token_accounts.get_token(collateral).map(MultiToken::from).unwrap().as_erc20().unwrap())
                    .unwrap();

            Self {
                owner: Lazy::new(Self::env().caller()),
                stable: Lazy::new(stable),
                collateral: Lazy::new(collateral),
                prices: Default::default(),
                oracles: Default::default(),
            }
        }

        /// Creates the contract without querying the respective erc20 tokens and token-accounts for
        /// data.
        #[ink(constructor)]
        pub fn new_raw(stable: TokenData, collateral: TokenData) -> Self {
            let stable = TokenSpec::from_data(stable);
            let collateral = TokenSpec::from_data(collateral);

            Self {
                owner: Lazy::new(Self::env().caller()),
                stable: Lazy::new(stable),
                collateral: Lazy::new(collateral),
                prices: Default::default(),
                oracles: Default::default(),
            }
        }

        /// Obtains the current price bucket.
        ///
        /// # Arguments
        ///
        /// * [SubmitPriceRequest](crate::models::SubmitPriceRequest): request specifying the oracle.
        #[ink(message)]
        pub fn get_price_bucket(&self, token: Ticker) -> Option<PriceBucket> { self.prices.get(&token).cloned() }

        /// Sets a weighted average of the current asset.
        ///
        /// # Restrictions
        ///
        /// May only be called by registered and active oracles.
        ///
        /// # Arguments
        ///
        /// * Ticker: the symbol of the token-accounts registered erc20 token.
        #[ink(message)]
        pub fn submit_price(&mut self, request: SubmitPriceRequest) -> Result<PriceBucket, SubmitPriceError> {
            let oracle = self.ensure_is_oracle()?;

            if oracle.state.is_disallowed() {
                return Err(SubmitPriceError::AuthzError(OracleError));
            }

            let token = request.token;
            let bucket = self.prices.entry(token.clone()).or_insert_with(|| PriceBucket {
                token: token.clone(),
                prices: Default::default(),
                volumes: Default::default(),
            });

            bucket.prices.insert(oracle.address, request.price);
            bucket.volumes.insert(oracle.address, request.volume);
            let bucket = bucket.clone();

            self.env().emit_event(PriceSubmitted::from(PriceSubmittedOutput { oracle: oracle.address, ticker: token }));
            Ok(bucket)
        }

        /// Obtains the current price of the ticker by weighted average of oracle data.
        ///
        /// # Arguments
        ///
        /// * Ticker: the symbol of the token-accounts registered erc20 token.
        #[ink(message)]
        pub fn get_price(&self, token: Ticker) -> Result<i128, GetPriceError> {
            let bucket = self.get_price_bucket(token).ok_or(GetPriceError::BucketNotFound)?;

            let (total, sum) = bucket.prices.iter().fold((0, 0): (i128, i128), |(total, sum), (address, &price)| {
                let volume = bucket.volumes.get(address).unwrap(); // Can only fail if a price was added without a volume.
                (total.saturating_add((*volume).into()), sum.saturating_add(price.saturating_mul(*volume).into()))
            });

            sum.checked_div(total).ok_or_else(|| GetPriceError::math_error("checked division of sum / total errored"))
        }

        /// Obtains the oracle. Is `None` if not registered.
        #[ink(message)]
        pub fn get_oracle(&self, address: AccountId) -> Option<Oracle> { self.oracles.get(&address).cloned() }

        /// Adds a new oracle to the set of allowed oracles. The oracle's state is set to `Allowed`.
        ///
        /// # Restrictions
        ///
        /// May only be called by the oracle owner.
        ///
        /// # Arguments
        ///
        /// * [RegisterOracleRequest](crate::models::RegisterOracleRequest): request specifying the oracle.
        #[ink(message)]
        pub fn register_oracle(&mut self, request: RegisterOracleRequest) -> Result<Oracle, RegisterOracleError> {
            self.ensure_is_owner()?;

            if self.get_oracle(request.address).is_some() {
                return Err(RegisterOracleError::AlreadyExists);
            }

            let oracle = Oracle { address: request.address, name: request.name, state: OracleState::Allowed };

            self.oracles.insert(request.address, oracle.clone());

            self.env().emit_event(OracleRegistered::from(OracleRegisteredOutput { oracle: oracle.clone() }));

            Ok(oracle)
        }

        /// Change the current state of the oracle.
        ///
        /// # Restrictions
        ///
        /// May only be called by the oracle owner.
        ///
        /// # Arguments
        ///
        /// * [UpdateOracleStateRequest](crate::models::UpdateOracleStateRequest): request specifying the new oracle state.
        #[ink(message)]
        pub fn update_oracle_state(
            &mut self,
            request: UpdateOracleStateRequest,
        ) -> Result<Oracle, UpdateOracleStateError> {
            self.ensure_is_owner()?;

            let oracle = self.oracles.get_mut(&request.address).ok_or(UpdateOracleStateError::DoesNotExist)?;
            oracle.state = request.state;

            let oracle = oracle.clone();

            self.env().emit_event(OracleStateUpdated::from(OracleStateUpdatedOutput {
                oracle: oracle.address,
                state: oracle.state.clone(),
            }));

            Ok(oracle)
        }

        fn ensure_is_oracle(&self) -> Result<Oracle, OracleError> {
            self.oracles.get(&self.env().caller()).cloned().ok_or(OracleError)
        }

        fn ensure_is_owner(&self) -> Result<AccountId, OwnerError> {
            if self.env().caller() != *self.owner {
                Err(OwnerError)
            } else {
                Ok(*self.owner)
            }
        }

        /// Swaps pUSD for Privi based on oracle provided prices.
        ///
        /// # Arguments
        ///
        /// * [ConvertRequest](crate::models::ConvertRequest): request specifying the conversion
        #[ink(message)]
        pub fn convert_to_privi(&mut self, request: ConvertRequest) -> Result<(), ConvertError> {
            self.convert(request, self.stable.clone(), self.collateral.clone())
        }

        /// Swaps Privi for pUSD based on oracle provided prices.
        ///
        /// # Arguments
        ///
        /// * [ConvertRequest](crate::models::ConvertRequest): request specifying the conversion
        #[ink(message)]
        pub fn convert_to_pusd(&mut self, request: ConvertRequest) -> Result<(), ConvertError> {
            self.convert(request, self.collateral.clone(), self.stable.clone())
        }

        fn convert(&self, request: ConvertRequest, mut from: TokenSpec, mut to: TokenSpec) -> Result<(), ConvertError> {
            let from_price =
                Decimal::from_i128_with_scale(self.get_price(from.ticker.clone())?, from.decimal_count.into());
            let to_price = Decimal::from_i128_with_scale(self.get_price(to.ticker.clone())?, to.decimal_count.into());

            if from_price.is_zero() || to_price.is_zero() {
                return Err(ConvertError::TokenValueIsZero);
            }

            let amount = compute_conversion(from_price, to_price, request.amount)?;

            from.erc20.burn_from(request.address, request.amount)?;
            to.erc20.mint(request.address, amount)?;
            self.env().emit_event(Conversion::from(ConversionOutput {
                from: from.ticker,
                to: to.ticker,
                caller: request.address,
            }));
            Ok(())
        }
    }

    fn compute_conversion(from: Decimal, to: Decimal, amount: Balance) -> Result<Balance, GetPriceError> {
        use rust_decimal::prelude::ToPrimitive;

        let ratio = from.checked_div(to).ok_or_else(|| GetPriceError::math_error("computing the ratio errored"))?;

        let amount = ratio * Decimal::from(amount);
        Ok(amount.to_u128().unwrap())
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use ink_lang as ink;
        use rust_decimal_macros::dec;

        #[ink::test]
        fn test_compute_conversion() {
            assert_eq!(compute_conversion(10.into(), 1.into(), 10).unwrap(), 100);
            assert_eq!(compute_conversion(10.into(), 1.into(), 1).unwrap(), 10);
        }
    }
}
