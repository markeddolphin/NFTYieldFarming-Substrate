#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

pub use crate::erc721::{Erc721, Error, TokenId, TokenInfo};
pub type Result<T> = core::result::Result<T, Error>;

#[ink::contract]
mod erc721 {
    use super::*;
    use ink_prelude::vec::Vec;
    use ink_storage::{
        collections::{hashmap::Entry, HashMap as StorageHashMap},
        traits::{PackedLayout, SpreadLayout},
    };
    use scale::{Decode, Encode};

    #[derive(Debug, Clone, PartialEq, Eq, scale::Encode, scale::Decode, SpreadLayout, PackedLayout)]
    #[cfg_attr(feature = "std", derive(::scale_info::TypeInfo, ::ink_storage::traits::StorageLayout))]
    pub struct TokenInfo {
        metadata: Vec<u8>,
    }

    /// A token ID.
    pub type TokenId = u64;

    #[ink(storage)]
    #[derive(Default)]
    pub struct Erc721 {
        /// Next Token Id
        next_token_id: u64,
        /// Mapping from TokenId to TokenInfo
        token_infos_by_id: StorageHashMap<TokenId, TokenInfo>,
        /// Mapping from token to owner.
        owners_by_token_id: StorageHashMap<TokenId, AccountId>,
        /// Mapping from owner to number of owned tokens.
        token_counts_by_account_id: StorageHashMap<AccountId, u64>,
        /// Mapping from token to approvals users.
        approvals_by_token_id: StorageHashMap<TokenId, AccountId>,
        /// Mapping from owner to operator approvals.
        operator_approvals: StorageHashMap<(AccountId, AccountId), bool>,
    }

    #[derive(Encode, Decode, Debug, PartialEq, Eq, Copy, Clone, err_derive::Error)]
    #[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
    pub enum Error {
        /// Cannot fetch Account Id
        #[error(display = "Cannot fetch Account Id")]
        CannotFetchValue,
        /// Maximum of Token reached
        #[error(display = "Maximum of Token reached")]
        TokenIdOverflow,
        /// Token Id not found
        #[error(display = "Token Id not found")]
        TokenNotFound,
        /// Account is not the Token owner
        #[error(display = "Account is not the Token owner")]
        NotOwner,
        /// Token id already existing (an Account is already set as owner)
        #[error(display = "Token id already existing (an Account is already set as owner)")]
        TokenExists,
        /// The caller is not allowed
        #[error(display = "The caller is not allowed")]
        NotAllowed,
        /// The caller is not an approved user
        #[error(display = "The caller is not an approved user")]
        NotApproved,
        /// Cannot insert the caller as approved user
        #[error(display = "Cannot insert the caller as approved user")]
        CannotInsert,
        /// Cannot remove the caller as approved user
        #[error(display = "Cannot remove the caller as approved user")]
        CannotRemove,
    }

    /// Event emitted when a token transfer occurs.
    #[ink(event)]
    pub struct Transfer {
        #[ink(topic)]
        from: Option<AccountId>,
        #[ink(topic)]
        to: Option<AccountId>,
        #[ink(topic)]
        id: TokenId,
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

    impl Erc721 {
        /// Creates a new ERC721 token contract.
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                next_token_id: 0,
                token_infos_by_id: Default::default(),
                owners_by_token_id: Default::default(),
                token_counts_by_account_id: Default::default(),
                approvals_by_token_id: Default::default(),
                operator_approvals: Default::default(),
            }
        }

        /// Returns the balance of the owner.
        ///
        /// This represents the amount of unique tokens the owner has.
        #[ink(message)]
        pub fn balance_of(&self, owner: AccountId) -> u64 { self.balance_of_or_zero(&owner) }

        /// Returns the owner of the token.
        #[ink(message)]
        pub fn owner_of(&self, id: TokenId) -> Option<AccountId> { self.owners_by_token_id.get(&id).cloned() }

        /// Returns the metadata of the token.
        #[ink(message)]
        pub fn token_info_of(&self, id: TokenId) -> Option<TokenInfo> { self.token_infos_by_id.get(&id).cloned() }

        /// Transfers the token from the caller to the given destination.
        #[ink(message)]
        pub fn transfer(&mut self, destination: AccountId, id: TokenId) -> Result<()> {
            let caller = self.env().caller();
            self.transfer_token_from(&caller, &destination, id)?;
            Ok(())
        }

        /// Transfer approved or owned token.
        #[ink(message)]
        pub fn transfer_from(&mut self, from: AccountId, to: AccountId, id: TokenId) -> Result<()> {
            self.transfer_token_from(&from, &to, id)?;
            Ok(())
        }

        /// Creates a new token.
        #[ink(message)]
        pub fn mint(&mut self, recipient: AccountId) -> Result<TokenId> {
            self.mint_with_metadata(recipient, Vec::new())
        }

        /// Creates a new token with metadata.
        #[ink(message)]
        pub fn mint_with_metadata(&mut self, recipient: AccountId, metadata: Vec<u8>) -> Result<TokenId> {
            let Self {
                next_token_id,
                token_infos_by_id: tokens,
                owners_by_token_id: token_owner,
                token_counts_by_account_id: owned_tokens_count,
                ..
            } = self;

            let token_id = get_next_token_id(*next_token_id)?;

            // Save token_id
            *next_token_id = token_id;

            // Create Token Info
            let token_info = TokenInfo { metadata };

            // Insert token Info
            tokens.insert(token_id, token_info);

            // Increase token count of to / owner of the minted Token
            let entry = owned_tokens_count.entry(recipient);
            increase_counter_of(entry);

            // Insert the caller as the owner of the minted Token
            token_owner.insert(token_id, recipient);

            self.env().emit_event(Transfer {
                from: Some(AccountId::from([0x0; 32])),
                to: Some(recipient),
                id: token_id,
            });

            Ok(token_id)
        }

        /// Deletes an existing token. Only the owner can burn the token.
        #[ink(message)]
        pub fn burn(&mut self, id: TokenId) -> Result<()> { self._burn_from(self.env().caller(), id) }

        /// Deletes an existing token from `account`. Requires approval and ownership.
        #[ink(message)]
        pub fn burn_from(&mut self, account: AccountId, id: TokenId) -> Result<()> { self._burn_from(account, id) }

        /// Internal implementation of both `burn` and `burn_from`
        fn _burn_from(&mut self, account: AccountId, id: TokenId) -> Result<()> {
            let caller = self.env().caller();

            if caller != account && !self.approved_or_owner(Some(caller), id) {
                return Err(Error::NotApproved);
            }
            if *self.owners_by_token_id.get(&id).ok_or(Error::TokenNotFound)? != account {
                return Err(Error::NotOwner);
            }

            decrease_counter_of(&mut self.token_counts_by_account_id, &account)?;
            self.owners_by_token_id.take(&id);
            self.env().emit_event(Transfer { from: Some(account), to: Some(AccountId::from([0x0; 32])), id });

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

        /// Transfers token `id` `from` the sender to the `to` AccountId.
        fn transfer_token_from(&mut self, from: &AccountId, to: &AccountId, id: TokenId) -> Result<()> {
            let caller = self.env().caller();

            if !self.exists(id) {
                return Err(Error::TokenNotFound);
            };
            if !self.approved_or_owner(Some(caller), id) {
                return Err(Error::NotApproved);
            };

            self.clear_approval(id)?;
            self.remove_token_from(from, id)?;
            self.add_token_to(to, id)?;

            self.env().emit_event(Transfer { from: Some(*from), to: Some(*to), id });
            Ok(())
        }

        /// Removes token `id` from the owner.
        fn remove_token_from(&mut self, from: &AccountId, id: TokenId) -> Result<()> {
            let Self { owners_by_token_id: token_owner, token_counts_by_account_id: owned_tokens_count, .. } = self;

            let occupied = match token_owner.entry(id) {
                Entry::Vacant(_) => return Err(Error::TokenNotFound),
                Entry::Occupied(occupied) => occupied,
            };

            decrease_counter_of(owned_tokens_count, from)?;
            occupied.remove_entry();

            Ok(())
        }

        /// Adds the token `id` to the `to` AccountID.
        fn add_token_to(&mut self, to: &AccountId, id: TokenId) -> Result<()> {
            let Self { owners_by_token_id: token_owner, token_counts_by_account_id: owned_tokens_count, .. } = self;

            let vacant_token_owner = match token_owner.entry(id) {
                Entry::Vacant(vacant) => vacant,
                Entry::Occupied(_) => return Err(Error::TokenExists),
            };
            if *to == AccountId::from([0x0; 32]) {
                return Err(Error::NotAllowed);
            };

            let entry = owned_tokens_count.entry(*to);
            increase_counter_of(entry);
            vacant_token_owner.insert(*to);

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

        // Returns the total number of tokens from an account.
        fn balance_of_or_zero(&self, of: &AccountId) -> u64 { *self.token_counts_by_account_id.get(of).unwrap_or(&0) }

        /// Removes existing approval from token `id`.
        fn clear_approval(&mut self, id: TokenId) -> Result<()> {
            if !self.approvals_by_token_id.contains_key(&id) {
                return Ok(());
            };
            self.approvals_by_token_id.take(&id);
            Ok(())

            // TODO: It seems like this is supposed to return an error if the approval cannot be cleared, but the
            // code would never trigger the error, and the test does not expect the error?

            // self.approvals_by_token_id.take(&id) {
            //     Some(_res) => Ok(()),
            //     None => Err(Error::CannotRemove),
            // }
        }

        /// Gets an operator on other Account's behalf.
        fn approved_for_all(&self, owner: AccountId, operator: AccountId) -> bool {
            *self.operator_approvals.get(&(owner, operator)).unwrap_or(&false)
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

        /// Approves or disapproves the operator for all tokens of the caller.
        #[ink(message)]
        pub fn set_approval_for_all(&mut self, to: AccountId, approved: bool) -> Result<()> {
            self.approve_for_all(to, approved)?;
            Ok(())
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

        /// Returns true if token `id` exists or false if it does not.
        fn exists(&self, id: TokenId) -> bool {
            self.owners_by_token_id.get(&id).is_some() && self.owners_by_token_id.contains_key(&id)
        }
    }

    // Get the next token Id
    #[allow(dead_code)]
    fn get_next_token_id(current_id: TokenId) -> Result<TokenId> {
        current_id.checked_add(1).ok_or(Error::TokenIdOverflow)
    }

    /// Decrease token counter from the `of` AccountId.
    #[allow(dead_code)]
    fn decrease_counter_of(hmap: &mut StorageHashMap<AccountId, u64>, of: &AccountId) -> Result<()> {
        let count = (*hmap).get_mut(of).ok_or(Error::CannotFetchValue)?;
        *count -= 1;
        Ok(())
    }

    /// Increase token counter from the `of` AccountId.
    #[allow(dead_code)]
    fn increase_counter_of(entry: Entry<AccountId, u64>) { entry.and_modify(|v| *v += 1).or_insert(1); }

    /// Unit tests
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
            let mut erc721 = Erc721::new();
            // Token 1 does not exists.
            assert_eq!(erc721.owner_of(1), None);
            // Alice does not owns tokens.
            assert_eq!(erc721.balance_of(accounts.alice), 0);
            // Create token Id 1.
            assert_eq!(erc721.mint(accounts.alice), Ok(1));
            // Alice owns 1 token.
            assert_eq!(erc721.balance_of(accounts.alice), 1);
        }

        #[ink::test]
        fn transfer_works() {
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>().expect("Cannot get accounts");
            // Create a new contract instance.
            let mut erc721 = Erc721::new();
            // Create token Id 1 for Alice
            assert_eq!(erc721.mint(accounts.alice), Ok(1));
            // Alice owns token 1
            assert_eq!(erc721.balance_of(accounts.alice), 1);
            // Bob does not owns any token
            assert_eq!(erc721.balance_of(accounts.bob), 0);
            // The first Transfer event takes place
            assert_eq!(1, ink_env::test::recorded_events().count());
            // Alice transfers token 1 to Bob
            assert_eq!(erc721.transfer(accounts.bob, 1), Ok(()));
            // The second Transfer event takes place
            assert_eq!(2, ink_env::test::recorded_events().count());
            // Bob owns token 1
            assert_eq!(erc721.balance_of(accounts.bob), 1);
        }

        #[ink::test]
        fn invalid_transfer_should_fail() {
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>().expect("Cannot get accounts");
            // Create a new contract instance.
            let mut erc721 = Erc721::new();
            // Transfer token fails if it does not exists.
            assert_eq!(erc721.transfer(accounts.bob, 2), Err(Error::TokenNotFound));
            // Token Id 2 does not exists.
            assert_eq!(erc721.owner_of(2), None);
            // Create token Id 1.
            assert_eq!(erc721.mint(accounts.alice), Ok(1));
            // Alice owns 1 token.
            assert_eq!(erc721.balance_of(accounts.alice), 1);
            // Token Id 1 is owned by Alice.
            assert_eq!(erc721.owner_of(1), Some(accounts.alice));
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
            assert_eq!(erc721.transfer(accounts.eve, 1), Err(Error::NotApproved));
        }

        #[ink::test]
        fn approved_transfer_works() {
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>().expect("Cannot get accounts");
            // Create a new contract instance.
            let mut erc721 = Erc721::new();
            // Create token Id 1.
            assert_eq!(erc721.mint(accounts.alice), Ok(1));
            // Token Id 1 is owned by Alice.
            assert_eq!(erc721.owner_of(1), Some(accounts.alice));
            // Approve token Id 1 transfer for Bob on behalf of Alice.
            assert_eq!(erc721.approve(accounts.bob, 1), Ok(()));
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
            assert_eq!(erc721.transfer_from(accounts.alice, accounts.eve, 1), Ok(()));
            // TokenId 1 is owned by Eve.
            assert_eq!(erc721.owner_of(1), Some(accounts.eve));
            // Alice does not owns tokens.
            assert_eq!(erc721.balance_of(accounts.alice), 0);
            // Bob does not owns tokens.
            assert_eq!(erc721.balance_of(accounts.bob), 0);
            // Eve owns 1 token.
            assert_eq!(erc721.balance_of(accounts.eve), 1);
        }

        #[ink::test]
        fn approved_for_all_works() {
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>().expect("Cannot get accounts");
            // Create a new contract instance.
            let mut erc721 = Erc721::new();
            // Create token Id 1.
            assert_eq!(erc721.mint(accounts.alice), Ok(1));
            // Create token Id 2.
            assert_eq!(erc721.mint(accounts.alice), Ok(2));
            // TokenId 1 is owned by Alice.
            assert_eq!(erc721.owner_of(1), Some(accounts.alice));
            // TokenId 2 is owned by Alice.
            assert_eq!(erc721.owner_of(2), Some(accounts.alice));
            // Alice owns 2 tokens.
            assert_eq!(erc721.balance_of(accounts.alice), 2);
            // Approve token Id 1 transfer for Bob on behalf of Alice.
            assert_eq!(erc721.set_approval_for_all(accounts.bob, true), Ok(()));
            // Bob is an approved operator for Alice
            assert_eq!(erc721.is_approved_for_all(accounts.alice, accounts.bob), true);
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
            assert_eq!(erc721.transfer_from(accounts.alice, accounts.eve, 1), Ok(()));
            // TokenId 1 is owned by Eve.
            assert_eq!(erc721.owner_of(1), Some(accounts.eve));
            // Alice owns 1 token.
            assert_eq!(erc721.balance_of(accounts.alice), 1);
            // Bob transfers token Id 2 from Alice to Eve.
            assert_eq!(erc721.transfer_from(accounts.alice, accounts.eve, 2), Ok(()));
            // Bob does not owns tokens.
            assert_eq!(erc721.balance_of(accounts.bob), 0);
            // Eve owns 2 tokens.
            assert_eq!(erc721.balance_of(accounts.eve), 2);
            // Get back to the parent execution context.
            ink_env::test::pop_execution_context();
            // Remove operator approval for Bob on behalf of Alice.
            assert_eq!(erc721.set_approval_for_all(accounts.bob, false), Ok(()));
            // Bob is not an approved operator for Alice.
            assert_eq!(erc721.is_approved_for_all(accounts.alice, accounts.bob), false);
        }

        #[ink::test]
        fn not_approved_transfer_should_fail() {
            let accounts = test_utils::default_accounts();
            let mut erc721 = Erc721::new();

            // Mint token to alice
            erc721.mint(accounts.alice).unwrap();
            assert_eq!(erc721.balance_of(accounts.alice), 1);

            // Eve and Bob do not own tokens
            assert_eq!(erc721.balance_of(accounts.bob), 0);
            assert_eq!(erc721.balance_of(accounts.eve), 0);

            // Eve is not an approved operator by Alice.
            test_utils::set_caller(accounts.eve);
            assert_eq!(erc721.transfer_from(accounts.alice, accounts.frank, 1), Err(Error::NotApproved));

            // Alice owns 1 token.
            assert_eq!(erc721.balance_of(accounts.alice), 1);
            // Eve does not owns tokens.
            assert_eq!(erc721.balance_of(accounts.eve), 0);
        }

        #[ink::test]
        fn burn_works() {
            let accounts = test_utils::default_accounts();
            let mut erc721 = Erc721::new();

            // Cannot burn nonexistent token
            assert_eq!(erc721.burn(1), Err(Error::TokenNotFound));

            // Mint token to Alice
            erc721.mint(accounts.alice).unwrap();
            assert_eq!(erc721.balance_of(accounts.alice), 1);
            assert_eq!(erc721.owner_of(1), Some(accounts.alice));

            // Destroy the token
            erc721.burn(1).unwrap();
            assert_eq!(erc721.balance_of(accounts.alice), 0);
            assert_eq!(erc721.owner_of(1), None);
        }

        #[ink::test]
        fn burn_from_works() {
            let accounts = test_utils::default_accounts();
            let mut erc721 = Erc721::new();

            // mint token to bob
            erc721.mint(accounts.bob).unwrap();

            // alice cannot burn
            assert_eq!(erc721.burn_from(accounts.bob, 1).unwrap_err(), Error::NotApproved);

            // Approve alice
            test_utils::set_caller(accounts.bob);
            erc721.approve(accounts.alice, 1);

            // now alice can burn
            test_utils::set_caller(accounts.alice);
            erc721.burn_from(accounts.bob, 1).unwrap();
        }

        #[ink::test]
        fn burn_fails_not_owner() {
            let accounts =
                ink_env::test::default_accounts::<ink_env::DefaultEnvironment>().expect("Cannot get accounts");
            // Create a new contract instance.
            let mut erc721 = Erc721::new();
            // Create token Id 1 for Alice
            assert_eq!(erc721.mint(accounts.alice), Ok(1));
            // Try burning this token with a different account
            set_sender(accounts.eve);
            assert_eq!(erc721.burn(1), Err(Error::NotOwner));
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
