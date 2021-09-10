//! Contains the code for the contract

use super::*;

/// Contains the contract code
#[ink::contract]
mod social_token {
    use super::*;
    use crate::amm::AmmType;
    use contract_utils::math::{BalanceExt, DecimalExt};
    use erc20::Erc20;
    use ink_storage::collections::HashMap;

    // ============= Events

    /// Emitted when the social token is bought
    #[ink(event)]
    pub struct Bought {
        /// The amount of tokens that were bought
        #[ink(topic)]
        amount: Balance,
        /// The total cost of the tokens
        #[ink(topic)]
        cost: Balance,
    }

    /// Emitted when the social token is sold
    #[ink(event)]
    pub struct Sold {
        #[ink(topic)]
        amount: Balance,
        #[ink(topic)]
        reward: Balance,
    }

    /// Emitted when the social token is withdrawn
    #[ink(event)]
    pub struct Withdrew {
        #[ink(topic)]
        amount: Balance,
    }

    /// Emitted when the social token is airdropped
    #[ink(event)]
    pub struct Airdropped {
        #[ink(topic)]
        user: AccountId,
        #[ink(topic)]
        amount: Balance,
    }

    /// Emitted when the funding token is withdrawn
    #[ink(event)]
    pub struct WithdrewFundingToken {
        #[ink(topic)]
        amount: Balance,
    }

    // ======== Storage

    /// Storage for the social token
    #[ink(storage)]
    pub struct SocialToken {
        /// Type of bonding curve.
        amm_type: AmmType,
        /// Spread charged for each trade. This accumulates on the smart contract and is to be divided between holders.
        trading_spread: Balance,
        /// The name of the token.
        token_name: String,
        /// The symbol of the token.
        token_symbol: String,
        /// Initial supply minted on the smart contract. This can be withdrawn by the social token creator at any moment. Also it can be airdropped and distributed to anyone else.
        initial_supply: Balance,
        /// Supply targeted by the user to be on the market (it can be more, this is just a target).
        target_supply: Balance,
        /// Price at which the social token will be trading when this TargetSupply is reached.
        target_price: Balance,
        /// The contract id for the ERC-20 token accepted for the AMM model (for example, DAI)
        funding_token_id: AccountId,
        /// The creation date
        creation_date: Timestamp,
        /// Mapping from owner to number of owned token.
        balances: HashMap<AccountId, Balance>,
        /// Amount minted minus amount burned
        supply_released: Balance,
        /// The total fee that has been accumulated from trading
        accumulated_trading_fee: Balance,
        /// The owner of the contract
        owner: AccountId,
    }

    impl SocialToken {
        /// Create a new instance.
        ///
        /// * `amm_type` - Type of bonding curve selected.
        /// * `trading_spread` - spread charged for each trade. This accumulates on the smart contract and is to be divided between holders.
        /// * `token_name` - The name of the token.
        /// * `token_symbol` - The symbol of the token.
        /// * `initial_supply` - initial supply minted on the smart contract. This can be withdraw by the social token creator at any moment. Also it can be airdropped and distributed to anyone else.
        /// * `target_supply` - supply targeted by the user to be on the market (it can be more, this is just a target).
        /// * `target_price` - price at which the social token will be trading when this TargetSupply is reached.
        /// * `funding_token_id` - The contract id for the ERC-20 token accepted for the AMM model (for example, DAI)
        #[ink(constructor)]
        #[allow(clippy::too_many_arguments)]
        pub fn new(
            amm_type: AmmType,
            trading_spread: Balance,
            token_name: String,
            token_symbol: String,
            initial_supply: Balance,
            target_supply: Balance,
            target_price: Balance,
            funding_token_id: AccountId,
        ) -> Self {
            let mut instance = Self {
                amm_type,
                trading_spread,
                token_name,
                token_symbol,
                initial_supply,
                target_supply,
                target_price,
                funding_token_id,
                creation_date: ink_env::block_timestamp::<DefaultEnvironment>().expect("could not get timestamp"),
                balances: Default::default(),
                supply_released: 0,
                accumulated_trading_fee: 0,
                owner: Self::env().caller(),
            };
            // store initial_supply at contract address
            instance.set_balance(Self::env().account_id(), initial_supply);

            instance
        }

        /// Buy `amount` social tokens. This function mints X social tokens and charges an amount Y of FundingToken determined by the bonding curve. Additionally, it charges a TradingSpread (on TradingToken).
        #[ink(message)]
        pub fn buy(&mut self, amount: Balance) -> Result<()> {
            let supply_released = self.supply_released + amount;
            let price = amm::price_for_mint(
                self.amm_type,
                supply_released.into_privi_decimal(),
                self.initial_supply.into_privi_decimal(),
                amount.into_privi_decimal(),
                self.target_price.into_privi_decimal(),
                self.target_supply.into_privi_decimal(),
            )?
            .into_privi_balance();

            // calculate the trading fee
            let trading_fee = price * self.trading_spread;

            // transfer the ERC-20 tokens
            let caller = self.env().caller();
            self.funding_token().transfer_from(caller, self.funding_token_account(), price + trading_fee)?;

            // mint the social tokens and update storage
            self.add_balance(caller, amount);
            self.accumulated_trading_fee += trading_fee;
            self.supply_released = supply_released;

            self.env().emit_event(Bought { amount, cost: price });

            Ok(())
        }

        /// Sells `amount` social tokens. This function burns X social tokens and gives an amount Y of FundingToken determined by the bonding curve. Additionally, it charges a trading spread (on TradingToken).
        #[ink(message)]
        pub fn sell(&mut self, amount: Balance) -> Result<()> {
            let supply_released = self.supply_released - amount;
            let reward = amm::reward_for_burn(
                self.amm_type,
                supply_released.into_privi_decimal(),
                self.initial_supply.into_privi_decimal(),
                amount.into_privi_decimal(),
                self.target_price.into_privi_decimal(),
                self.target_supply.into_privi_decimal(),
            )?
            .into_privi_balance();

            // calculate the trading fee
            let trading_fee = reward * self.trading_spread;

            // transfer the funding tokens
            let caller = self.env().caller();
            self.funding_token().transfer(caller, reward - trading_fee)?;

            // burn the social tokens and update storage
            self.set_balance(caller, self.balance_of(caller) - amount);
            self.accumulated_trading_fee += trading_fee;
            self.supply_released = supply_released;

            self.env().emit_event(Sold { amount, reward });
            Ok(())
        }

        /// Withdraws an `amount` of the initial supply to the owner's account. Can only be called by `owner`.
        #[ink(message)]
        pub fn withdraw(&mut self, amount: Balance) -> Result<()> {
            if self.initial_supply < amount {
                return Err(Error::InsufficientInitialSupplyBalance);
            }
            if self.env().caller() != self.owner {
                return Err(Error::InsufficientAccess);
            }

            self.initial_supply -= amount;

            // move amount from contract to owner
            self.subtract_balance(self.social_token_account(), amount);
            self.add_balance(self.owner, amount);

            self.env().emit_event(Withdrew { amount });
            Ok(())
        }

        /// Sends `amount` social tokens from initial supply to `to` account. Can only be called by `owner`.
        #[ink(message)]
        pub fn airdrop(&mut self, amount: Balance, to: AccountId) -> Result<()> {
            if self.initial_supply < amount {
                return Err(Error::InsufficientInitialSupplyBalance);
            }
            if self.env().caller() != self.owner {
                return Err(Error::InsufficientAccess);
            }
            self.initial_supply -= amount;
            self.subtract_balance(self.social_token_account(), amount);
            self.add_balance(to, amount);

            self.env().emit_event(Airdropped { user: to, amount });
            Ok(())
        }

        /// This function is called by the owner to withdraw some of the tokens accumulated by the trading activity.
        #[ink(message)]
        pub fn withdraw_funding_token(&mut self, amount: Balance) -> Result<()> {
            if self.accumulated_trading_fee < amount {
                return Err(Error::InsufficientTradingFeeBalance);
            }
            self.accumulated_trading_fee -= amount;
            self.funding_token().transfer(self.owner, amount)?;

            self.env().emit_event(WithdrewFundingToken { amount });
            Ok(())
        }

        /// The account that stores the social tokens
        fn social_token_account(&self) -> AccountId { self.env().account_id() }

        /// The account that stores the funding tokens
        fn funding_token_account(&self) -> AccountId { self.env().account_id() }
    }

    impl SocialToken {
        // Get the ERC-20 token
        fn funding_token(&self) -> Erc20 {
            cfg_if::cfg_if! {
                if #[cfg(target_arch = "wasm32")] {
                    use ink_env::call::FromAccountId;
                    Erc20::from_account_id(self.funding_token_id)
                } else {
                    unimplemented!();
                }
            }
        }

        /// Returns the account balance for the specified `owner` or `0` if the account does not exist
        fn balance_of(&self, owner: AccountId) -> Balance { self.balances.get(&owner).copied().unwrap_or(0) }

        /// Sets the balance of an account
        fn set_balance(&mut self, account: AccountId, value: Balance) { self.balances.insert(account, value); }

        /// Adds to the balance of an account
        fn add_balance(&mut self, account: AccountId, amount: Balance) {
            self.set_balance(account, self.balance_of(account) + amount);
        }

        /// Subtracts from the balance of an account
        fn subtract_balance(&mut self, account: AccountId, amount: Balance) {
            self.set_balance(account, self.balance_of(account) - amount);
        }
    }
}
