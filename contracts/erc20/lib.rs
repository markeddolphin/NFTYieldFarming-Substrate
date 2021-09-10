#![cfg_attr(not(feature = "std"), no_std)]
#![deny(unused_must_use)]

use contract_utils::env_exports::*;
use ink_lang as ink;
use scale::{Decode, Encode};

pub use contract::Erc20;

/// Error types
#[derive(Debug, Copy, Clone, PartialEq, Eq, Encode, Decode, err_derive::Error)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum Error {
    /// Not enough balance.
    #[error(display = "Not enough balance")]
    InsufficientBalance,
    /// Not enough allowance.
    #[error(display = "Not enough allowance.")]
    InsufficientAllowance,
    /// The zero address cannot be used for this operation
    #[error(display = "The zero address cannot be used for this operation")]
    ZeroAddressNotAllowed,
    /// A required role is missing for this operation
    #[error(display = "A required role is missing for this operation")]
    MissingRole,
    /// Transfers cannot be completed because they are paused
    #[error(display = "Transfers cannot be completed because they are paused")]
    TransfersPaused,
}

/// The ERC-20 result type.
pub type Result<T> = core::result::Result<T, Error>;

/// Trait implemented by all ERC-20 respecting smart contracts.
#[ink::trait_definition]
pub trait Erc20Base {
    /// Returns the total token supply.
    #[ink(message)]
    fn total_supply(&self) -> Balance;

    /// Returns the account balance for the specified `owner`.
    #[ink(message)]
    fn balance_of(&self, owner: AccountId) -> Balance;

    /// Returns the amount which `spender` is still allowed to withdraw from `owner`.
    #[ink(message)]
    fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance;

    /// Transfers `value` amount of tokens from the caller's account to account `to`.
    #[ink(message)]
    fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()>;

    /// Allows `spender` to withdraw from the caller's account multiple times, up to
    /// the `value` amount.
    #[ink(message)]
    fn approve(&mut self, spender: AccountId, value: Balance) -> Result<()>;

    /// Transfers `value` tokens on the behalf of `from` to the account `to`.
    #[ink(message)]
    fn transfer_from(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<()>;
}

/// Enables minting of coins
#[ink::contract]
pub mod contract {
    use super::*;
    use enumflags2::{bitflags, BitFlags};
    use ink_prelude::string::String;

    #[cfg(not(feature = "ink-as-dependency"))]
    use contract_utils::ZERO_ACCOUNT;
    #[cfg(not(feature = "ink-as-dependency"))]
    use ink_storage::{collections::HashMap, lazy::Lazy};

    /// An ERC-20 contract extended with burn, mint, and pause functionality.
    #[ink(storage)]
    pub struct Erc20 {
        /// Total token supply.
        total_supply: Lazy<Balance>,
        /// Mapping from owner to number of owned token.
        balances: HashMap<AccountId, Balance>,
        /// Mapping of the token amount which an account is allowed to withdraw
        /// from another account.
        allowances: HashMap<(AccountId, AccountId), Balance>,
        /// Roles for each account. Each value is stored as a raw BitFlags<Role>.
        roles: HashMap<AccountId, u8>,
        /// The pause state of the contract
        is_paused: bool,

        // optional data
        /// An optional name
        name: Lazy<Option<String>>,
        /// Optional symbol of the token
        symbol: Lazy<Option<String>>,
        /// Optional decimals of the token
        decimal_count: Lazy<Option<u8>>,
    }

    // ========= ERC20 ========

    /// Event emitted when a token transfer occurs.
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        #[ink(topic)]
        value: Balance,
    }

    /// Event emitted when an approval occurs that `spender` is allowed to withdraw
    /// up to the amount of `value` tokens from `owner`.
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        spender: AccountId,
        #[ink(topic)]
        value: Balance,
    }

    /// Event emitted when a minter is added.
    #[ink(event)]
    pub struct AddedMinter {
        #[ink(topic)]
        account: AccountId,
    }

    /// Event emitted when a minter is removed.
    #[ink(event)]
    pub struct RemovedMinter {
        #[ink(topic)]
        account: AccountId,
    }

    /// Event emitted when a minter is added.
    #[ink(event)]
    pub struct AddedBurner {
        #[ink(topic)]
        account: AccountId,
    }

    /// Event emitted when a minter is removed.
    #[ink(event)]
    pub struct RemovedBurner {
        #[ink(topic)]
        account: AccountId,
    }

    impl Erc20 {
        /// Creates a new ERC-20 contract with the specified initial supply.
        #[ink(constructor)]
        pub fn new(initial_supply: Balance) -> Self { Self::new_optional(initial_supply, None, None, None) }

        /// Create a new instance with additional optional arguments
        #[ink(constructor)]
        pub fn new_optional(
            initial_supply: Balance,
            name: Option<String>,
            symbol: Option<String>,
            decimal_count: Option<u8>,
        ) -> Self {
            let caller = Self::env().caller();
            let mut balances = HashMap::new();
            balances.insert(caller, initial_supply);

            let mut roles = HashMap::new();
            roles.insert(caller, RoleBitFlags::all().bits());

            let instance = Self {
                total_supply: Lazy::new(initial_supply),
                balances,
                allowances: HashMap::new(),
                roles,
                is_paused: false,
                name: Lazy::new(name),
                symbol: Lazy::new(symbol),
                decimal_count: Lazy::new(decimal_count),
            };
            Self::env().emit_event(Transfer { from: None, to: Some(caller), value: initial_supply });
            instance
        }

        /// Returns the total token supply.
        #[ink(message)]
        pub fn total_supply(&self) -> Balance { *self.total_supply }

        /// Returns the account balance for the specified `owner`.
        ///
        /// Returns `0` if the account is non-existent.
        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> Balance { self.balances.get(&owner).copied().unwrap_or(0) }

        /// Returns the amount which `spender` is allowed to withdraw from `owner`.
        ///
        /// Returns `0` if no allowance has been set `0`.
        #[ink(message)]
        pub fn allowance(&self, owner: AccountId, spender: AccountId) -> Balance {
            self.allowances.get(&(owner, spender)).copied().unwrap_or(0)
        }

        /// Transfers `value` amount of tokens from the caller's account to account `to`.
        ///
        /// On success a `Transfer` event is emitted.
        ///
        /// # Errors
        ///
        /// Returns `InsufficientBalance` error if there are not enough tokens on
        /// the caller's account balance.
        #[ink(message)]
        pub fn transfer(&mut self, to: AccountId, value: Balance) -> Result<()> {
            self._transfer_from_to(self.caller(), to, value)
        }

        /// Allows `spender` to withdraw from the caller's account multiple times, up to
        /// the `value` amount.
        ///
        /// If this function is called again it overwrites the current allowance with `value`.
        ///
        /// An `Approval` event is emitted.
        #[ink(message)]
        #[ink(selector = "0x681266a0")]
        pub fn approve(&mut self, spender: AccountId, value: Balance) -> Result<()> {
            let owner = self.env().caller();
            self.allowances.insert((owner, spender), value);
            self.env().emit_event(Approval { owner, spender, value });
            Ok(())
        }

        /// Transfers `value` tokens on the behalf of `from` to the account `to`.
        ///
        /// This can be used to allow a contract to transfer tokens on ones behalf and/or
        /// to charge fees in sub-currencies, for example.
        ///
        /// On success a `Transfer` event is emitted.
        ///
        /// # Errors
        ///
        /// Returns `InsufficientAllowance` error if there are not enough tokens allowed
        /// for the caller to withdraw from `from`.
        ///
        /// Returns `InsufficientBalance` error if there are not enough tokens on
        /// the the account balance of `from`.
        #[ink(message)]
        pub fn transfer_from(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<()> {
            let caller = self.env().caller();
            let allowance = self.allowance(from, caller);
            if allowance < value {
                return Err(Error::InsufficientAllowance);
            }
            self._transfer_from_to(from, to, value)?;
            self.allowances.insert((from, caller), allowance - value);
            Ok(())
        }
    }

    // ========== ACCESS CONTROL

    /// Roles grant an account permissions for certain operations
    #[bitflags]
    #[derive(Debug, Clone, Copy, Eq, PartialEq)]
    #[repr(u8)]
    pub enum Role {
        /// Can change roles (1)
        Admin = 0b_0000_0001,
        /// Can mint tokens (2)
        Minter = 0b_0000_0010,
        /// Can burn tokens (4)
        Burner = 0b_0000_0100,
        /// Can pause transfers (8)
        Pauser = 0b_0000_1000,
    }

    pub type RoleBitFlags = BitFlags<Role>;

    #[ink(impl)]
    impl Erc20 {
        /// Add a role to an account. Caller must have Admin role.
        fn add_roles(&mut self, caller: AccountId, account: AccountId, value: RoleBitFlags) -> Result<()> {
            if !self.get_roles(caller).contains(Role::Admin) {
                return Err(Error::MissingRole);
            }
            self.roles
                .entry(account)
                .and_modify(|x| *x = (unsafe { BitFlags::from_bits_unchecked(*x) } | value).bits())
                .or_insert(value.bits());
            Ok(())
        }

        /// Remove a role from an account. Caller must have Admin role.
        fn remove_roles(&mut self, caller: AccountId, account: AccountId, value: RoleBitFlags) -> Result<()> {
            if !self.get_roles(caller).contains(Role::Admin) {
                return Err(Error::MissingRole);
            }
            self.roles
                .entry(account)
                .and_modify(|x| *x = (unsafe { BitFlags::from_bits_unchecked(*x) } & !value).bits())
                .or_insert(0);
            Ok(())
        }

        /// Get the roles for this account
        fn get_roles(&self, account: AccountId) -> RoleBitFlags {
            unsafe { RoleBitFlags::from_bits_unchecked(self.roles.get(&account).copied().unwrap_or_default()) }
        }
    }

    /// Test adding, removing roles, including admin functionality
    #[ink::test]
    fn test_roles_and_admin() {
        let mut contract = test_utils::new_erc20(1);
        let accounts = test_utils::default_accounts();

        // alice should start out as admin
        assert!(contract.get_roles(accounts.alice).contains(Role::Admin));

        // Accounts have no roles by default
        assert_eq!(contract.get_roles(accounts.bob), BitFlags::empty());

        // Bob cannot change permissions because he's not Admin
        contract.add_roles(accounts.bob, accounts.alice, Role::Burner.into()).unwrap_err();
        contract.remove_roles(accounts.bob, accounts.alice, Role::Burner.into()).unwrap_err();

        // add two roles and verify
        contract.add_roles(accounts.alice, accounts.bob, Role::Admin | Role::Burner).unwrap();
        assert_eq!(contract.get_roles(accounts.bob), Role::Admin | Role::Burner);

        // remove one role, make sure the first is still there
        contract.remove_roles(accounts.alice, accounts.bob, Role::Burner.into()).unwrap();
        assert_eq!(contract.get_roles(accounts.bob), BitFlags::from_flag(Role::Admin));

        // Since bob is an admin, he can modify roles now
        contract.remove_roles(accounts.bob, accounts.alice, Role::Admin.into()).unwrap();
        contract.add_roles(accounts.alice, accounts.alice, Role::Admin.into()).unwrap_err();
        contract.add_roles(accounts.bob, accounts.alice, Role::Admin.into()).unwrap();
    }

    impl Erc20 {
        /// Sends `amount` coins to mint account
        #[ink(message)]
        #[ink(selector = "0xcfdd9aa2")]
        pub fn mint(&mut self, recipient: AccountId, amount: Balance) -> Result<()> {
            self._mint(self.caller(), recipient, amount)
        }

        /// Internal implementation of mint
        fn _mint(&mut self, caller: AccountId, recipient: AccountId, amount: Balance) -> Result<()> {
            // check if it's allowed
            if self.is_paused() {
                return Err(Error::TransfersPaused);
            }
            if recipient == ZERO_ACCOUNT {
                return Err(Error::ZeroAddressNotAllowed);
            }
            if !self.get_roles(caller).contains(Role::Minter) {
                return Err(Error::MissingRole);
            }

            // add to total supply
            let total_supply = self.total_supply();
            *self.total_supply = total_supply + amount;

            // add to account
            let balance = self.balance_of(recipient);
            self.set_balance(recipient, balance + amount);

            // emit event
            self.env().emit_event(Transfer { from: Some(ZERO_ACCOUNT), to: Some(recipient), value: amount });
            Ok(())
        }

        /// Add the Minter role to an account
        #[ink(message)]
        pub fn add_minter(&mut self, account: AccountId) -> Result<()> {
            self.add_roles(self.caller(), account, Role::Minter.into())?;
            self.env().emit_event(AddedMinter { account });
            Ok(())
        }

        /// Remove the Minter role from the account
        #[ink(message)]
        pub fn remove_minter(&mut self, account: AccountId) -> Result<()> {
            self.remove_roles(self.caller(), account, Role::Minter.into())?;
            self.env().emit_event(RemovedMinter { account });
            Ok(())
        }
    }

    /// Test all mint functionality
    #[ink::test]
    #[cfg(test)]
    fn test_mint() {
        let mut contract = test_utils::new_erc20(100);
        let accounts = test_utils::default_accounts();

        // alice should already be a minter
        assert!(contract.get_roles(accounts.alice).contains(Role::Minter));

        // Bob tries to mint coins to Alice. Should fail because he's not a Minter.
        contract._mint(accounts.bob, accounts.alice, 2).unwrap_err();

        // Alice makes Bob a minter.
        contract.add_roles(accounts.alice, accounts.bob, Role::Minter.into()).unwrap();

        // Bob should be able to mint coins to alice now
        let event_count = test_utils::recorded_event_count();
        contract._mint(accounts.bob, accounts.alice, 1).unwrap();
        assert_eq!(test_utils::recorded_event_count(), event_count + 1);
        assert_eq!(contract.balance_of(accounts.alice), 101);
        assert_eq!(contract.total_supply(), 101);

        // Alice removes Bob as a minter
        contract.remove_roles(accounts.alice, accounts.bob, Role::Minter.into()).unwrap();

        // Bob can no longer mint
        contract._mint(accounts.bob, accounts.alice, 2).unwrap_err();
    }

    // impl Burnable for Contract {
    impl Erc20 {
        /// Destroys `amount` tokens
        #[ink(message)]
        pub fn burn(&mut self, amount: Balance) -> Result<()> { self._burn_from(self.caller(), self.caller(), amount) }

        /// Internal implementation of burn
        pub fn _burn_from(&mut self, caller: AccountId, account: AccountId, amount: Balance) -> Result<()> {
            if self.is_paused() {
                return Err(Error::TransfersPaused);
            }

            if account == ZERO_ACCOUNT {
                return Err(Error::ZeroAddressNotAllowed);
            }
            if !self.get_roles(caller).contains(Role::Burner) {
                return Err(Error::MissingRole);
            }

            let balance = self.balance_of(account);
            if balance < amount {
                return Err(Error::InsufficientBalance);
            }

            if caller != account {
                let allowance = self.allowance(account, caller);
                if allowance < amount {
                    return Err(Error::InsufficientAllowance);
                }
                self.allowances.insert((account, caller), allowance.saturating_sub(amount));
            }

            // set new balance
            self.set_balance(account, balance.saturating_sub(amount));

            // reduce total supply
            let total_supply = self.total_supply();
            *self.total_supply = total_supply.saturating_sub(amount);

            self.env().emit_event(Transfer { from: Some(account), to: Some(ZERO_ACCOUNT), value: amount });
            Ok(())
        }

        #[ink(message)]
        #[ink(selector = "0x27212bbb")]
        pub fn burn_from(&mut self, account: AccountId, amount: Balance) -> Result<()> {
            self._burn_from(self.caller(), account, amount)
        }

        /// Add the burner role to an account
        #[ink(message)]
        pub fn add_burner(&mut self, account: AccountId) -> Result<()> {
            self.add_roles(self.caller(), account, Role::Burner.into())?;
            self.env().emit_event(AddedBurner { account });
            Ok(())
        }

        /// Remove the burner role from the account
        #[ink(message)]
        pub fn remove_burner(&mut self, account: AccountId) -> Result<()> {
            self.remove_roles(self.caller(), account, Role::Burner.into())?;
            self.env().emit_event(RemovedBurner { account });
            Ok(())
        }
    }

    /// Test all burn functionality
    #[ink::test]
    #[cfg(test)]
    fn test_burn() {
        let mut contract = test_utils::new_erc20(1000);
        let accounts = test_utils::default_accounts();

        // give some the coins to bob
        contract.transfer(accounts.bob, 100).unwrap();

        // alice should already be a burner
        assert!(contract.get_roles(accounts.alice).contains(Role::Burner));

        // Bob tries to burn coins. Should fail because he's not a Burner.
        contract._burn_from(accounts.bob, accounts.bob, 1).unwrap_err();

        // Alice makes Bob a Burner.
        contract.add_roles(accounts.alice, accounts.bob, Role::Burner.into()).unwrap();

        // Bob should be able to burn coins now
        let event_count = test_utils::recorded_event_count();
        contract._burn_from(accounts.bob, accounts.bob, 1).unwrap();
        assert_eq!(test_utils::recorded_event_count(), event_count + 1);
        assert_eq!(contract.balance_of(accounts.bob), 99);
        assert_eq!(contract.total_supply(), 999);

        // Alice removes Bob as a burner
        contract.remove_roles(accounts.alice, accounts.bob, Role::Burner.into()).unwrap();

        // Bob can no longer burn
        contract._burn_from(accounts.bob, accounts.bob, 1).unwrap_err();

        // Alice tries to burn Bob's coins, but she can't because no allowance
        contract._burn_from(accounts.alice, accounts.bob, 1).unwrap_err();

        test_utils::set_caller(accounts.bob);
        contract.approve(accounts.alice, 5).unwrap();
        contract._burn_from(accounts.alice, accounts.bob, 1).unwrap();
        assert_eq!(contract.allowance(accounts.bob, accounts.alice), 4);
    }

    /// An event emitted when the contract is paused
    #[ink(event)]
    #[derive(Default)]
    pub struct Paused;

    /// An event emitted when the contract is unpaused
    #[ink(event)]
    #[derive(Default)]
    pub struct Unpaused;

    // impl Pausable for Contract {
    impl Erc20 {
        /// Get the pause state
        #[ink(message)]
        pub fn is_paused(&self) -> bool { self.is_paused }

        /// Pauses the contract
        #[ink(message)]
        pub fn pause(&mut self) -> Result<()> { self._set_is_paused(self.caller(), true) }

        /// Unpauses the contract
        #[ink(message)]
        pub fn unpause(&mut self) -> Result<()> { self._set_is_paused(self.caller(), false) }

        /// Set the pause state (internal)
        fn _set_is_paused(&mut self, caller: AccountId, value: bool) -> Result<()> {
            if !self.get_roles(caller).contains(Role::Pauser) {
                return Err(Error::MissingRole);
            }
            self.is_paused = value;
            if value {
                self.env().emit_event(Paused::default());
            } else {
                self.env().emit_event(Unpaused::default());
            }
            Ok(())
        }

        /// Add the Pauser role to an account
        #[ink(message)]
        pub fn add_pauser(&mut self, account: AccountId) -> Result<()> {
            self.add_roles(self.caller(), account, Role::Pauser.into())
        }

        /// Remove the Pauser role from the account
        #[ink(message)]
        pub fn remove_pauser(&mut self, account: AccountId) -> Result<()> {
            self.remove_roles(self.caller(), account, Role::Pauser.into())
        }
    }

    #[ink::test]
    #[cfg(test)]
    fn test_pause() {
        let mut contract = test_utils::new_erc20(100);
        let accounts = test_utils::default_accounts();

        // make sure everything succeeds before pausing
        contract._transfer_from_to(accounts.alice, accounts.bob, 10).unwrap();
        contract.mint(accounts.alice, 100).unwrap();
        contract.burn(100).unwrap();

        // bob cannot pause
        contract._set_is_paused(accounts.bob, true).unwrap_err();

        // make bob a pauser
        contract.add_roles(accounts.alice, accounts.bob, Role::Pauser.into()).unwrap();

        // pause and now they should all fail
        let event_count = test_utils::recorded_event_count();
        contract._set_is_paused(accounts.bob, true).unwrap();
        assert_eq!(test_utils::recorded_event_count(), event_count + 1);

        contract._transfer_from_to(accounts.alice, accounts.bob, 10).unwrap_err();
        contract.mint(accounts.alice, 100).unwrap_err();
        contract.burn(100).unwrap_err();

        let event_count = test_utils::recorded_event_count();
        contract.unpause().unwrap();
        assert_eq!(test_utils::recorded_event_count(), event_count + 1);

        contract._transfer_from_to(accounts.alice, accounts.bob, 10).unwrap();
    }

    // ========== Optional Data
    impl Erc20 {
        /// The number of decimals
        #[ink(message)]
        pub fn decimal_count(&self) -> Option<u8> { *self.decimal_count }

        /// The name of the token
        #[ink(message)]
        pub fn name(&self) -> Option<String> { self.name.clone() }

        /// The symbol for the token
        #[ink(message)]
        pub fn symbol(&self) -> Option<String> { self.symbol.clone() }
    }

    #[ink(impl)]
    impl Erc20 {
        /// The caller of the contract
        fn caller(&self) -> AccountId {
            self.env().caller()
            // self.owner
        }

        /// Sets the balance of an account
        fn set_balance(&mut self, account: AccountId, value: Balance) { self.balances.insert(account, value); }

        /// Transfers `value` amount of tokens from the caller's account to account `to`.
        ///
        /// On success a `Transfer` event is emitted.
        ///
        /// # Errors
        ///
        /// Returns `InsufficientBalance` error if there are not enough tokens on
        /// the caller's account balance.
        fn _transfer_from_to(&mut self, from: AccountId, to: AccountId, value: Balance) -> Result<()> {
            if self.is_paused() {
                return Err(Error::TransfersPaused);
            }

            let from_balance = self.balance_of(from);
            if from_balance < value {
                return Err(Error::InsufficientBalance);
            }
            self.set_balance(from, from_balance - value);
            let to_balance = self.balance_of(to);
            self.set_balance(to, to_balance + value);
            self.env().emit_event(Transfer { from: Some(from), to: Some(to), value });
            Ok(())
        }
    }

    /// Unit tests.
    #[cfg(test)]
    mod tests {
        /// Imports all the definitions from the outer scope so we can use them here.
        use super::*;
        use ink_env::{
            hash::{Blake2x256, CryptoHash, HashOutput},
            Clear,
        };
        use ink_lang as ink;

        type Event = <Erc20 as ::ink_lang::BaseEvent>::Type;

        /// For calculating the event topic hash.
        struct PrefixedValue<'a, 'b, T> {
            pub prefix: &'a [u8],
            pub value: &'b T,
        }

        impl<X> scale::Encode for PrefixedValue<'_, '_, X>
        where
            X: scale::Encode,
        {
            #[inline]
            fn size_hint(&self) -> usize { self.prefix.size_hint() + self.value.size_hint() }

            #[inline]
            fn encode_to<T: scale::Output + ?Sized>(&self, dest: &mut T) {
                self.prefix.encode_to(dest);
                self.value.encode_to(dest);
            }
        }

        fn assert_transfer_event(
            event: &ink_env::test::EmittedEvent,
            expected_from: Option<AccountId>,
            expected_to: Option<AccountId>,
            expected_value: Balance,
        ) {
            let decoded_event = <Event as scale::Decode>::decode(&mut &event.data[..])
                .expect("encountered invalid contract event data buffer");
            if let Event::Transfer(Transfer { from, to, value }) = decoded_event {
                assert_eq!(from, expected_from, "encountered invalid Transfer.from");
                assert_eq!(to, expected_to, "encountered invalid Transfer.to");
                assert_eq!(value, expected_value, "encountered invalid Trasfer.value");
            } else {
                panic!("encountered unexpected event kind: expected a Transfer event")
            }
            fn encoded_into_hash<T>(entity: &T) -> Hash
            where
                T: scale::Encode,
            {
                let mut result = Hash::clear();
                let len_result = result.as_ref().len();
                let encoded = entity.encode();
                let len_encoded = encoded.len();
                if len_encoded <= len_result {
                    result.as_mut()[..len_encoded].copy_from_slice(&encoded);
                    return result;
                }
                let mut hash_output = <<Blake2x256 as HashOutput>::Type as Default>::default();
                <Blake2x256 as CryptoHash>::hash(&encoded, &mut hash_output);
                let copy_len = core::cmp::min(hash_output.len(), len_result);
                result.as_mut()[0..copy_len].copy_from_slice(&hash_output[0..copy_len]);
                result
            }

            let expected_topics = vec![
                encoded_into_hash(&PrefixedValue { prefix: b"", value: b"Erc20::Transfer" }),
                encoded_into_hash(&PrefixedValue { prefix: b"Erc20::Transfer::from", value: &expected_from }),
                encoded_into_hash(&PrefixedValue { prefix: b"Erc20::Transfer::to", value: &expected_to }),
                encoded_into_hash(&PrefixedValue { prefix: b"Erc20::Transfer::value", value: &expected_value }),
            ];
            for (n, (actual_topic, expected_topic)) in event.topics.iter().zip(expected_topics).enumerate() {
                let topic = actual_topic.decode::<Hash>().expect("encountered invalid topic encoding");
                assert_eq!(topic, expected_topic, "encountered invalid topic at {}", n);
            }
        }

        /// The default constructor does its job.
        #[ink::test]
        fn test_new() {
            // Constructor works.
            let initial_supply = 100;
            let contract = Erc20::new(initial_supply);

            // The `BaseErc20` trait has indeed been implemented.
            assert_eq!(contract.total_supply(), initial_supply);

            // Transfer event triggered during initial construction.
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(1, emitted_events.len());

            assert_transfer_event(&emitted_events[0], None, Some(AccountId::from([0x01; 32])), 100);
        }

        /// The total supply was applied.
        #[ink::test]
        fn test_total_supply() {
            // Constructor works.
            let erc20 = test_utils::new_erc20(100);
            // Transfer event triggered during initial construction.
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_transfer_event(&emitted_events[0], None, Some(AccountId::from([0x01; 32])), 100);
            // Get the token total supply.
            assert_eq!(erc20.total_supply(), 100);
        }

        /// Get the actual balance of an account.
        #[ink::test]
        fn test_balance_of() {
            // Constructor works
            let erc20 = test_utils::new_erc20(100);
            // Transfer event triggered during initial construction
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_transfer_event(&emitted_events[0], None, Some(AccountId::from([0x01; 32])), 100);
            let accounts = test_utils::default_accounts();
            // Alice owns all the tokens on deployment
            assert_eq!(erc20.balance_of(accounts.alice), 100);
            // Bob does not own tokens
            assert_eq!(erc20.balance_of(accounts.bob), 0);
        }

        #[ink::test]
        fn test_transfer() {
            // Constructor works.
            let mut erc20 = test_utils::new_erc20(100);
            // Transfer event triggered during initial construction.
            let accounts = test_utils::default_accounts();

            assert_eq!(erc20.balance_of(accounts.bob), 0);
            // Alice transfers 10 tokens to Bob.
            assert_eq!(erc20._transfer_from_to(accounts.alice, accounts.bob, 10), Ok(()));
            // Bob owns 10 tokens.
            assert_eq!(erc20.balance_of(accounts.bob), 10);

            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 2);
            // Check first transfer event related to ERC-20 instantiation.
            assert_transfer_event(&emitted_events[0], None, Some(AccountId::from([0x01; 32])), 100);
            // Check the second transfer event relating to the actual trasfer.
            assert_transfer_event(
                &emitted_events[1],
                Some(AccountId::from([0x01; 32])),
                Some(AccountId::from([0x02; 32])),
                10,
            );
        }

        #[ink::test]
        fn test_invalid_transfer_should_fail() {
            // Constructor works.
            let mut erc20 = test_utils::new_erc20(100);
            let accounts = test_utils::default_accounts();

            assert_eq!(erc20.balance_of(accounts.alice), 100);
            assert_eq!(erc20.balance_of(accounts.bob), 0);

            // Bob fails to transfer 10 tokens to Eve.
            assert_eq!(erc20._transfer_from_to(accounts.bob, accounts.eve, 10), Err(Error::InsufficientBalance));
            // Alice owns all the tokens.
            assert_eq!(erc20.balance_of(accounts.alice), 100);
            assert_eq!(erc20.balance_of(accounts.bob), 0);
            assert_eq!(erc20.balance_of(accounts.eve), 0);

            // Transfer event triggered during initial construction.
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 1);
            assert_transfer_event(&emitted_events[0], None, Some(AccountId::from([0x01; 32])), 100);
        }

        #[ink::test]
        fn test_transfer_from() {
            // Constructor works.
            let mut erc20 = test_utils::new_erc20(100);
            // Transfer event triggered during initial construction.
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>().expect("Cannot get accounts");

            // Bob fails to transfer tokens owned by Alice.
            assert_eq!(erc20.transfer_from(accounts.alice, accounts.eve, 10), Err(Error::InsufficientAllowance));
            // Alice approves Bob for token transfers on her behalf.
            assert_eq!(erc20.approve(accounts.bob, 10), Ok(()));

            // The approve event takes place.
            assert_eq!(ink_env::test::recorded_events().count(), 2);

            // Get contract address.
            let callee = ink_env::account_id::<ink_env::DefaultEnvironment>().unwrap_or_else(|_| [0x0; 32].into());
            // Create call.
            let mut data = ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4])); // balance_of
            data.push_arg(&accounts.bob);
            // Push the new execution context to set Bob as caller.
            ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(
                accounts.bob,
                callee,
                1000000,
                1000000,
                data,
            );

            // Bob transfers tokens from Alice to Eve.
            assert_eq!(erc20.transfer_from(accounts.alice, accounts.eve, 10), Ok(()));
            // Eve owns tokens.
            assert_eq!(erc20.balance_of(accounts.eve), 10);

            // Check all transfer events that happened during the previous calls:
            let emitted_events = ink_env::test::recorded_events().collect::<Vec<_>>();
            assert_eq!(emitted_events.len(), 3);
            assert_transfer_event(&emitted_events[0], None, Some(AccountId::from([0x01; 32])), 100);
            // The second event `emitted_events[1]` is an Approve event that we skip checking.
            assert_transfer_event(
                &emitted_events[2],
                Some(AccountId::from([0x01; 32])),
                Some(AccountId::from([0x05; 32])),
                10,
            );
        }

        #[ink::test]
        fn test_allowance_must_not_change_on_failed_transfer() {
            let mut contract = test_utils::new_erc20(100);
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>().expect("Cannot get accounts");

            // Alice approves Bob for token transfers on her behalf.
            let alice_balance = contract.balance_of(accounts.alice);
            let initial_allowance = alice_balance + 2;
            assert_eq!(contract.approve(accounts.bob, initial_allowance), Ok(()));

            // Get contract address.
            let callee = ink_env::account_id::<ink_env::DefaultEnvironment>().unwrap_or_else(|_| [0x0; 32].into());
            // Create call.
            let mut data = ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4])); // balance_of
            data.push_arg(&accounts.bob);
            // Push the new execution context to set Bob as caller.
            ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(
                accounts.bob,
                callee,
                1000000,
                1000000,
                data,
            );

            // Bob tries to transfer tokens from Alice to Eve.
            let emitted_events_before_count = ink_env::test::recorded_events().count();
            assert_eq!(
                contract.transfer_from(accounts.alice, accounts.eve, alice_balance + 1),
                Err(Error::InsufficientBalance)
            );
            // Allowance must have stayed the same
            assert_eq!(contract.allowance(accounts.alice, accounts.bob), initial_allowance);
            // No more events must have been emitted
            assert_eq!(emitted_events_before_count, ink_env::test::recorded_events().count());
        }
    }

    #[cfg(test)]
    mod test_utils {
        use super::*;
        pub use contract_utils::test_utils::*;

        /// Returns a new contract with alice as the owner
        pub fn new_erc20(initial_supply: Balance) -> Erc20 { Erc20::new(initial_supply) }
    }
}
