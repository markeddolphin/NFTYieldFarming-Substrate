// Right now all metadata (JSON) is store on chain
// Later : Update token info to Store an Uri
#![allow(clippy::new_without_default)]
#![cfg_attr(not(feature = "std"), no_std)]

pub use crate::erc1155::{Erc1155, Error};
use ink_lang as ink;

pub type Result<T> = core::result::Result<T, Error>;

#[ink::contract]
mod erc1155 {
    use super::*;
    use ink_prelude::vec::Vec;
    use ink_storage::{
        collections::HashMap as StorageHashMap,
        traits::{PackedLayout, SpreadLayout},
    };
    use scale::{Decode, Encode};


    #[derive(Debug, Clone, PartialEq, Eq, Encode, Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo, ::ink_storage::traits::StorageLayout))]
    pub struct TokenInfo {
        metadata: Vec<u8>,
    }

    pub type TokenId = u64;

    #[ink(storage)]
    pub struct Erc1155 {
        /// Next Token Id
        next_token_id: u64,
        /// Mapping from TokenId to TokensInfo (Metadata)
        tokens_by_id: StorageHashMap<TokenId, TokenInfo>,
        /// Mapping from token to owner.
        owners_by_token_id: StorageHashMap<TokenId, AccountId>,
        /// Balances of each account for each Token
        balances_by_account_id: StorageHashMap<(AccountId, TokenId), Balance>,
        /// Mapping from token to approvals users.
        approvals_by_token_id: StorageHashMap<TokenId, AccountId>,
        /// Mapping from owner to operator approvals.
        operator_approvals: StorageHashMap<(AccountId, AccountId), bool>,
    }

    #[derive(Encode, Decode, Debug, PartialEq, Eq, Copy, Clone, err_derive::Error)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// The caller is not allowed
        #[error(display = "The caller is not allowed")]
        NotAllowed,
        /// The caller is not an approved user
        #[error(display = "The caller is not an approved user")]
        NotApproved,
        /// Maximum of Token reached        
        #[error(display = "Maximum of Token reached")]
        TokenIdOverflow,
        /// Token Id not found        
        #[error(display = "Token Id not found")]
        TokenNotFound,
        /// Account is not the Token owner        
        #[error(display = "Account is not the Token owner")]
        NotOwner,
        /// Cannot fect Account Id
        #[error(display = "Cannot fect Account Id")]
        CannotFetchValue,
        /// Amount to burn is higer than Balance        
        #[error(display = "Amount to burn is higer than Balance")]
        BurnAmountExceedsBalance,
        /// Insufficient Balance
        #[error(display = "Insufficient Balance")]
        InsufficientBalance,
        /// ids and values array length must match.
        #[error(display = "ids and values array length must match.")]
        ArraysLengthNotEqual,
        /// Cannot insert the caller as approved user        
        #[error(display = "Cannot insert the caller as approved user")]
        CannotInsert,
    }

    /// Event emitted when a token transfer occurs.
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        #[ink(topic)]
        token_id: u64,
        amount: Balance,
    }

    /// Event emitted when a token approve occurs.
    #[ink(event)]
    pub struct Approval {
        #[ink(topic)]
        from: AccountId,
        #[ink(topic)]
        to: AccountId,
        #[ink(topic)]
        id: TokenId,
    }

    /// Event emitted when an operator is enabled or disabled for an owner.
    /// The operator can manage all NFTs of the owner.
    #[ink(event)]
    pub struct ApprovalForAll {
        #[ink(topic)]
        owner: AccountId,
        #[ink(topic)]
        operator: AccountId,
        approved: bool,
    }

    impl Erc1155 {
        /// Creates a new ERC1155 token contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                next_token_id: 0,
                tokens_by_id: Default::default(),
                owners_by_token_id: Default::default(),
                balances_by_account_id: Default::default(),
                approvals_by_token_id: Default::default(),
                operator_approvals: Default::default(),
            }
        }

        /// Creates a new token.
        #[ink(message)]
        pub fn mint(&mut self, recipient: AccountId, amount: Balance, metadata: Vec<u8>) -> Result<()> {
            let caller = self.env().caller();
            if caller == AccountId::from([0x0; 32]) {
                return Err(Error::NotAllowed);
            };
            let Self {
                next_token_id,
                tokens_by_id: tokens,
                owners_by_token_id: token_owner,
                balances_by_account_id: balances,
                ..
            } = self;

            let token_id = get_next_token_id(*next_token_id)?;

            // Save token_id
            *next_token_id = token_id;

            // Create Token Info
            let token_info = TokenInfo { metadata };

            // Store the new Token Id and its Token Info
            tokens.insert(token_id, token_info);

            // Store the recipient as the Token owner (map via TokenId)
            token_owner.insert(token_id, recipient);

            // Store the amount of Token that is created
            // if amount = 0 then it is an NFT
            // if amount is >0 then it is a fungible token
            balances.insert((recipient, token_id), amount);

            self.env().emit_event(Transfer {
                from: Some(AccountId::from([0x0; 32])),
                to: Some(recipient),
                token_id,
                amount,
            });

            Ok(())
        }

        /// Burns amount token of TokenId from an account
        #[ink(message)]
        pub fn burn(&mut self, id: TokenId, amount: Balance) -> Result<()> {
            self._burn_from(self.env().caller(), id, amount)
        }

        /// Burns amount token of TokenId from `account`
        #[ink(message)]
        pub fn burn_from(&mut self, account: AccountId, id: TokenId, amount: Balance) -> Result<()> {
            self._burn_from(account, id, amount)
        }

        /// Internal implementation of both `burn` and `burn_from`
        pub fn _burn_from(&mut self, account: AccountId, id: TokenId, amount: Balance) -> Result<()> {
            let caller = self.env().caller();

            if caller != account && !self.approved_or_owner(Some(caller), id) {
                return Err(Error::NotApproved);
            }
            if *self.owners_by_token_id.get(&id).ok_or(Error::TokenNotFound)? != account {
                return Err(Error::NotOwner);
            }


            reduce_balance_of(&mut self.balances_by_account_id, account, id, amount)?;

            self.env().emit_event(Transfer {
                from: Some(account),
                to: Some(AccountId::from([0x0; 32])),
                token_id: id,
                amount,
            });

            Ok(())
        }

        /// Approve the passed AccountId to transfer the specified token on behalf of the message's sender.
        fn approve_for(&mut self, to: &AccountId, id: TokenId) -> Result<()> {
            let caller = self.env().caller();

            let owner = self.owner_of(id);
            if !(owner == Some(caller) || self.approved_for_all(owner.expect("Error with AccountId"), caller)) {
                return Err(Error::NotAllowed);
            };
            if *to == AccountId::from([0x0; 32]) {
                return Err(Error::NotAllowed);
            };

            if self.approvals_by_token_id.insert(id, *to).is_some() {
                return Err(Error::CannotInsert);
            };

            self.env().emit_event(Approval { from: caller, to: *to, id });
            Ok(())
        }

        /// Transfers the token from the caller to the given destination.
        #[ink(message)]
        pub fn transfer(&mut self, destination: AccountId, id: TokenId, amount: Balance) -> Result<()> {
            let caller = self.env().caller();
            self.transfer_token_from_to(caller, destination, id, amount)?;
            Ok(())
        }

        /// Transfer approved or owned token.
        #[ink(message)]
        pub fn transfer_from(&mut self, from: AccountId, to: AccountId, id: TokenId, amount: Balance) -> Result<()> {
            self.transfer_token_from_to(from, to, id, amount)?;
            Ok(())
        }

        /// Transfers tokens from the caller to the given destination. Batch Transfer
        #[ink(message)]
        pub fn batch_transfer(
            &mut self,
            destination: AccountId,
            ids: Vec<TokenId>,
            amounts: Vec<Balance>,
        ) -> Result<()> {
            if ids.len() != amounts.len() {
                return Err(Error::ArraysLengthNotEqual);
            }
            let caller = self.env().caller();
            for i in 0..ids.len() {
                self.transfer_token_from_to(caller, destination, ids[i], amounts[i])?;
            }
            Ok(())
        }

        /// Transfers token `id` `from` the sender to the `to` AccountId.
        fn transfer_token_from_to(
            &mut self,
            from: AccountId,
            to: AccountId,
            token_id: TokenId,
            amount: Balance,
        ) -> Result<()> {
            let caller = self.env().caller();

            if !self.exists(token_id) {
                return Err(Error::TokenNotFound);
            };

            if !self.approved_or_owner(Some(caller), token_id) {
                return Err(Error::NotApproved);
            };

            let Self { balances_by_account_id: balances, .. } = self;

            reduce_balance_of(balances, from, token_id, amount)?;

            increase_balance_of(balances, to, token_id, amount)?;

            self.env().emit_event(Transfer { from: Some(from), to: Some(to), token_id, amount });

            Ok(())
        }

        /// Returns the owner of the token.
        #[ink(message)]
        pub fn owner_of(&self, id: TokenId) -> Option<AccountId> { self.owners_by_token_id.get(&id).cloned() }

        /// Returns the metadata of the token.
        #[ink(message)]
        pub fn token_info_of(&self, id: TokenId) -> Option<TokenInfo> { self.tokens_by_id.get(&id).cloned() }

        /// Returns the balance of the owner.
        /// This represents the amount the owner has forn a given TokenId.
        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId, id: TokenId) -> Balance { self.balance_of_or_zero(&owner, id) }

        /// Returns the total amount of a given Token from an account.
        fn balance_of_or_zero(&self, of: &AccountId, id: TokenId) -> Balance {
            let balance = *self.balances_by_account_id.get(&(*of, id)).unwrap_or(&0);
            balance
        }

        /// Returns true if token `id` exists or false if it does not.
        fn exists(&self, id: TokenId) -> bool {
            self.owners_by_token_id.get(&id).is_some() && self.owners_by_token_id.contains_key(&id)
        }

        /// Returns true if the AccountId `from` is the owner of token `id`
        /// or it has been approved on behalf of the token `id` owner.
        fn approved_or_owner(&self, from: Option<AccountId>, id: TokenId) -> bool {
            let owner = self.owner_of(id);
            from != Some(AccountId::from([0x0; 32]))
                && (from == owner
                    || from == self.approvals_by_token_id.get(&id).cloned()
                    || self.approved_for_all(owner.expect("Error with AccountId"), from.expect("Error with AccountId")))
        }

        /// Gets an operator on other Account's behalf.
        fn approved_for_all(&self, owner: AccountId, operator: AccountId) -> bool {
            *self.operator_approvals.get(&(owner, operator)).unwrap_or(&false)
        }

        /// Approves or disapproves the operator for all tokens of the caller.
        #[ink(message)]
        pub fn set_approval_for_all(&mut self, to: AccountId, approved: bool) -> Result<()> {
            self.approve_for_all(to, approved)?;
            Ok(())
        }

        /// Approves or disapproves the operator to transfer all tokens of the caller.
        fn approve_for_all(&mut self, to: AccountId, approved: bool) -> Result<()> {
            let caller = self.env().caller();
            if to == caller {
                return Err(Error::NotAllowed);
            }

            self.env().emit_event(ApprovalForAll { owner: caller, operator: to, approved });

            if self.approved_for_all(caller, to) {
                let status = self.operator_approvals.get_mut(&(caller, to)).ok_or(Error::CannotFetchValue)?;
                *status = approved;
                Ok(())
            } else {
                match self.operator_approvals.insert((caller, to), approved) {
                    Some(_) => Err(Error::CannotInsert),
                    None => Ok(()),
                }
            }
        }

        /// Approves the account to transfer the specified token on behalf of the caller.
        #[ink(message)]
        pub fn approve(&mut self, to: AccountId, id: TokenId) -> Result<()> {
            self.approve_for(&to, id)?;
            Ok(())
        }

        /// Returns `true` if the operator is approved by the owner.
        #[ink(message)]
        pub fn is_approved_for_all(&self, owner: AccountId, operator: AccountId) -> bool {
            self.approved_for_all(owner, operator)
        }
    }

    /// Get the next token Id
    #[allow(dead_code)]
    fn get_next_token_id(current_id: TokenId) -> Result<TokenId> {
        current_id.checked_add(1).ok_or(Error::TokenIdOverflow)
    }

    /// Reduce the balance of AccountId for amount of the TokenId
    #[allow(dead_code)]
    fn reduce_balance_of(
        balances: &mut StorageHashMap<(AccountId, TokenId), Balance>,
        of: AccountId,
        id: TokenId,
        amount: Balance,
    ) -> Result<()> {
        let entry_balance = (*balances).get_mut(&(of, id)).ok_or(Error::CannotFetchValue)?;
        if *entry_balance < amount {
            return Err(Error::InsufficientBalance);
        }
        *entry_balance -= amount;
        Ok(())
    }

    /// Increase the balance of AccountId for amount of the TokenId
    #[allow(dead_code)]
    fn increase_balance_of(
        balances: &mut StorageHashMap<(AccountId, TokenId), Balance>,
        of: AccountId,
        id: TokenId,
        amount: Balance,
    ) -> Result<()> {
        if !(*balances).contains_key(&(of, id)) {
            // This account has no balance for this TokenId
            // Store this balance with the amount provided
            (*balances).insert((of, id), amount);
        } else {
            let entry_balance = (*balances).get_mut(&(of, id)).ok_or(Error::CannotFetchValue)?;
            *entry_balance += amount;
        }

        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use contract_utils::test_utils;
        use ink_env::{call, test};
        use ink_lang as ink;

        #[ink::test]
        fn mint_works() {
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>().expect("Cannot get accounts");
            // Create a new contract instance.
            let mut erc1155 = Erc1155::new();
            // Alice does not owns tokens.
            assert_eq!(erc1155.balance_of(accounts.alice, 1), 0);
            // Create token Id 1.
            assert_eq!(erc1155.mint(accounts.alice, 10000, "Some JSON".as_bytes().to_vec()), Ok(()));
            // Alice owns 10000 of tokenId 1.
            assert_eq!(erc1155.balance_of(accounts.alice, 1), 10000);
        }

        #[ink::test]
        fn burn_works() {
            let accounts = test_utils::default_accounts();
            let mut erc1155 = Erc1155::new();

            // Try burning a non existent token
            assert_eq!(erc1155.burn(1, 10000), Err(Error::TokenNotFound));

            // Create token Id 1 for Alice
            erc1155.mint(accounts.alice, 10000, vec![13]).unwrap();
            assert_eq!(erc1155.balance_of(accounts.alice, 1), 10000);
            assert_eq!(erc1155.owner_of(1), Some(accounts.alice));

            // Destroy some of token
            erc1155.burn(1, 5000).unwrap();
            assert_eq!(erc1155.balance_of(accounts.alice, 1), 5000);
        }

        #[ink::test]
        fn burn_from_works() {
            let accounts = test_utils::default_accounts();
            let mut erc1155 = Erc1155::new();

            // mint token to bob
            erc1155.mint(accounts.bob, 10, vec![56]).unwrap();

            // alice cannot burn
            assert_eq!(erc1155.burn_from(accounts.bob, 1, 5).unwrap_err(), Error::NotApproved);

            // Approve alice
            test_utils::set_caller(accounts.bob);
            erc1155.approve(accounts.alice, 1);

            // now alice can burn
            test_utils::set_caller(accounts.alice);
            erc1155.burn_from(accounts.bob, 1, 5).unwrap();
            assert_eq!(erc1155.balance_of(accounts.bob, 1), 5);
        }

        #[ink::test]
        fn burn_fails_not_owner() {
            let accounts = test_utils::default_accounts();
            // Create a new contract instance.
            let mut erc1155 = Erc1155::new();
            // Create token Id 1 for Alice
            assert_eq!(erc1155.mint(accounts.alice, 10000, "Some JSON".as_bytes().to_vec()), Ok(()));
            // Try burning this token with a different account
            set_sender(accounts.eve);
            assert_eq!(erc1155.burn(1, 5000), Err(Error::NotOwner));
        }

        #[ink::test]
        fn transfer_works() {
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>().expect("Cannot get accounts");
            // Create a new contract instance.
            let mut erc1155 = Erc1155::new();
            // Create token Id 1 for Alice
            assert_eq!(erc1155.mint(accounts.alice, 10000, "Some JSON".as_bytes().to_vec()), Ok(()));
            // Alice owns token 1
            assert_eq!(erc1155.balance_of(accounts.alice, 1), 10000);
            // Bob does not owns any token
            assert_eq!(erc1155.balance_of(accounts.bob, 1), 0);
            // The first Transfer event takes place
            assert_eq!(1, ink_env::test::recorded_events().count());
            // Alice transfers token 1 to Bob
            assert_eq!(erc1155.transfer(accounts.bob, 1, 5000), Ok(()));
            // The second Transfer event takes place
            assert_eq!(2, ink_env::test::recorded_events().count());
            // Bob owns token 1
            assert_eq!(erc1155.balance_of(accounts.bob, 1), 5000);
        }

        #[ink::test]
        fn batch_transfer_works() {
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>().expect("Cannot get accounts");
            // Create a new contract instance.
            let mut erc1155 = Erc1155::new();
            // Create token Id 1 for Alice
            assert_eq!(erc1155.mint(accounts.alice, 10000, "Some JSON 1".as_bytes().to_vec()), Ok(()));
            // Create token Id 2 for Alice
            assert_eq!(erc1155.mint(accounts.alice, 10000, "Some JSON 2".as_bytes().to_vec()), Ok(()));
            // Create token Id 3 for Alice
            assert_eq!(erc1155.mint(accounts.alice, 10000, "Some JSON 3".as_bytes().to_vec()), Ok(()));
            // Alice owns token 1
            assert_eq!(erc1155.balance_of(accounts.alice, 1), 10000);
            // Alice owns token 2
            assert_eq!(erc1155.balance_of(accounts.alice, 2), 10000);
            // Alice owns token 3
            assert_eq!(erc1155.balance_of(accounts.alice, 3), 10000);
            // Bob does not owns any token
            assert_eq!(erc1155.balance_of(accounts.bob, 1), 0);
            // Three Transfer events took place
            assert_eq!(3, ink_env::test::recorded_events().count());
            // Alice transfers all tokens to Bob
            assert_eq!(erc1155.batch_transfer(accounts.bob, vec![1, 2, 3], vec![10000, 5000, 1000]), Ok(()));
            // Bob owns 10000 of token 1
            assert_eq!(erc1155.balance_of(accounts.bob, 1), 10000);
            // Bob owns 5000 of token 2
            assert_eq!(erc1155.balance_of(accounts.bob, 2), 5000);
            // Bob owns 1000 of token 3
            assert_eq!(erc1155.balance_of(accounts.bob, 3), 1000);
        }

        #[ink::test]
        fn invalid_transfer_should_fail() {
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>().expect("Cannot get accounts");
            // Create a new contract instance.
            let mut erc1155 = Erc1155::new();
            // Transfer token fails if it does not exists.
            assert_eq!(erc1155.transfer(accounts.bob, 2, 5000), Err(Error::TokenNotFound));
            // Token Id 2 does not exists.
            assert_eq!(erc1155.owner_of(2), None);
            // Create token Id 1.
            assert_eq!(erc1155.mint(accounts.alice, 10000, "Some JSON".as_bytes().to_vec()), Ok(()));
            // Alice owns 10000 of token Id 1.
            assert_eq!(erc1155.balance_of(accounts.alice, 1), 10000);
            // Get contract address
            let callee = ink_env::account_id::<ink_env::DefaultEnvironment>().unwrap_or([0x0; 32].into());
            // Create call
            let mut data = ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4])); // balance_of
            data.push_arg(&accounts.bob);
            // Push the new execution context to set Bob as caller
            ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(
                accounts.bob,
                callee,
                1000000,
                1000000,
                data,
            );
            // Bob cannot transfer not owned tokens.
            assert_eq!(erc1155.transfer(accounts.eve, 1, 5000), Err(Error::NotApproved));
        }

        #[ink::test]
        fn approved_transfer_works() {
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>().expect("Cannot get accounts");
            // Create a new contract instance.
            let mut erc1155 = Erc1155::new();
            // Create token Id 1.
            assert_eq!(erc1155.mint(accounts.alice, 10000, "Some JSON".as_bytes().to_vec()), Ok(()));
            // Token Id 1 is owned by Alice.
            assert_eq!(erc1155.owner_of(1), Some(accounts.alice));
            // Approve token Id 1 transfer for Bob on behalf of Alice.
            assert_eq!(erc1155.approve(accounts.bob, 1), Ok(()));
            // Get contract address.
            let callee = ink_env::account_id::<ink_env::DefaultEnvironment>().unwrap_or([0x0; 32].into());
            // Create call
            let mut data = ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4])); // balance_of
            data.push_arg(&accounts.bob);
            // Push the new execution context to set Bob as caller
            ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(
                accounts.bob,
                callee,
                1000000,
                1000000,
                data,
            );
            // Bob transfers token Id 1 from Alice to Eve.
            assert_eq!(erc1155.transfer_from(accounts.alice, accounts.eve, 1, 5000), Ok(()));
            // Alice owns 5000 of token Id 1
            assert_eq!(erc1155.balance_of(accounts.alice, 1), 5000);
            // Bob does not owns tokens.
            assert_eq!(erc1155.balance_of(accounts.bob, 1), 0);
            // Eve owns 5000 of Token 1
            assert_eq!(erc1155.balance_of(accounts.eve, 1), 5000);
        }

        #[ink::test]
        fn approved_for_all_works() {
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>().expect("Cannot get accounts");
            // Create a new contract instance.
            let mut erc1155 = Erc1155::new();
            // Create token Id 1.
            assert_eq!(erc1155.mint(accounts.alice, 10000, "Some JSON 1".as_bytes().to_vec()), Ok(()));
            // Create token Id 2.
            assert_eq!(erc1155.mint(accounts.alice, 10000, "Some JSON 2".as_bytes().to_vec()), Ok(()));
            // TokenId 1 is owned by Alice.
            assert_eq!(erc1155.owner_of(1), Some(accounts.alice));
            // TokenId 2 is owned by Alice.
            assert_eq!(erc1155.owner_of(2), Some(accounts.alice));
            // Approve token Id 1 transfer for Bob on behalf of Alice.
            assert_eq!(erc1155.set_approval_for_all(accounts.bob, true), Ok(()));
            // Bob is an approved operator for Alice
            assert_eq!(erc1155.is_approved_for_all(accounts.alice, accounts.bob), true);
            // Get contract address.
            let callee = ink_env::account_id::<ink_env::DefaultEnvironment>().unwrap_or([0x0; 32].into());
            // Create call
            let mut data = ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4])); // balance_of
            data.push_arg(&accounts.bob);
            // Push the new execution context to set Bob as caller
            ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(
                accounts.bob,
                callee,
                1000000,
                1000000,
                data,
            );
            // Bob transfers token Id 1 from Alice to Eve.
            assert_eq!(erc1155.transfer_from(accounts.alice, accounts.eve, 1, 5000), Ok(()));
            // Alice owns 5000 of Token Id1
            assert_eq!(erc1155.balance_of(accounts.alice, 1), 5000);
            // Bob transfers token Id 2 from Alice to Eve.
            assert_eq!(erc1155.transfer_from(accounts.alice, accounts.eve, 2, 10000), Ok(()));
            // Bob does not owns tokens.
            assert_eq!(erc1155.balance_of(accounts.bob, 1), 0);
            assert_eq!(erc1155.balance_of(accounts.bob, 2), 0);
            // Eve owns 5000 of Token 1
            assert_eq!(erc1155.balance_of(accounts.eve, 1), 5000);
            // Eve owns 10000 of Token 2
            assert_eq!(erc1155.balance_of(accounts.eve, 2), 10000);
            // Get back to the parent execution context.
            ink_env::test::pop_execution_context();
            // Remove operator approval for Bob on behalf of Alice.
            assert_eq!(erc1155.set_approval_for_all(accounts.bob, false), Ok(()));
            // Bob is not an approved operator for Alice.
            assert_eq!(erc1155.is_approved_for_all(accounts.alice, accounts.bob), false);
        }

        #[ink::test]
        fn not_approved_transfer_should_fail() {
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>().expect("Cannot get accounts");
            // Create a new contract instance.
            let mut erc1155 = Erc1155::new();
            // Create token Id 1.
            assert_eq!(erc1155.mint(accounts.alice, 10000, "Some JSON".as_bytes().to_vec()), Ok(()));
            // Alice owns 10 000 of tokenId 1
            assert_eq!(erc1155.balance_of(accounts.alice, 1), 10000);
            // Bob does not owns tokenId 1
            assert_eq!(erc1155.balance_of(accounts.bob, 1), 0);
            // Eve does not owns tokenId 1
            assert_eq!(erc1155.balance_of(accounts.eve, 1), 0);
            // Get contract address.
            let callee = ink_env::account_id::<ink_env::DefaultEnvironment>().unwrap_or([0x0; 32].into());
            // Create call
            let mut data = ink_env::test::CallData::new(ink_env::call::Selector::new([0x00; 4])); // balance_of
            data.push_arg(&accounts.bob);
            // Push the new execution context to set Eve as caller
            ink_env::test::push_execution_context::<ink_env::DefaultEnvironment>(
                accounts.eve,
                callee,
                1000000,
                1000000,
                data,
            );
            // Eve is not an approved operator by Alice.
            assert_eq!(erc1155.transfer_from(accounts.alice, accounts.eve, 1, 1000), Err(Error::NotApproved));
            // Alice owns 10 000 of tokenId 1
            assert_eq!(erc1155.balance_of(accounts.alice, 1), 10000);
            // Bob does not owns tokenId 1
            assert_eq!(erc1155.balance_of(accounts.bob, 1), 0);
            // Eve does not owns tokenId 1
            assert_eq!(erc1155.balance_of(accounts.eve, 1), 0);
        }

        fn set_sender(sender: AccountId) {
            let callee = ink_env::account_id::<ink_env::DefaultEnvironment>().unwrap_or([0x0; 32].into());
            test::push_execution_context::<Environment>(
                sender,
                callee,
                1000000,
                1000000,
                test::CallData::new(call::Selector::new([0x00; 4])), // dummy
            );
        }
    }
}
