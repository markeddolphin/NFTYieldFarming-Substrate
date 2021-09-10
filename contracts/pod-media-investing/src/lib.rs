#![cfg_attr(not(feature = "std"), no_std)]

use ink_lang as ink;

pub mod errors;
pub mod models;

#[ink::contract]
mod pod_media_investing {
    use crate::{
        errors::Error,
        models::{CreateInvestingPodRequest, InvestingPodState, InvestingPodStatus, RegisterMediaRequest},
    };
    use contract_utils::AccountIdExt;
    use erc20::Erc20;
    use ink_storage::collections::Vec as StorageVec;
    use media::{models::MediaId, MediaStorage};

    type Result<T> = core::result::Result<T, Error>;

    /// An InvestingPod is a media pod which goes through three states:
    ///
    /// `Formation`: The creator uploads and registers media.
    /// `Investing`: Users may purchase pod-tokens at the exchange rate of `funding_token_price`
    ///              until `funding_target` is reached.
    /// `Trading`: Through an `AMM`, pod-tokens become purchasable.
    #[ink(storage)]
    pub struct InvestingPod {
        creator: AccountId,
        amm_address: AccountId,
        amm_curve: ::amm::Curve,
        spread: u32,
        funding_token_price: Balance,
        funding_token: Erc20,
        pod_token: Erc20,
        funding_target: Balance,
        max_price: Balance,
        max_supply: Balance,
        created_at: Timestamp,
        funding_date: Timestamp,
        media_ids: StorageVec<MediaId>,
        state: InvestingPodState,
        media: MediaStorage,
    }

    impl InvestingPod {
        /// Create a new InvestingPod. Note that this constructor performs validations and will panic
        /// if `CreateInvestingPodRequest` is invalid. (Constructors cannot return Result<Self>).
        #[ink(constructor)]
        pub fn new(request: CreateInvestingPodRequest) -> Self {
            let now = Self::env().block_timestamp();
            request.validate(now).expect("validating request failed");

            // This will panic on overflows or divide by zero. request.validate already checks for
            // divide by zero, and the caller should not be passing in numbers that can overflow cause
            // overflows
            let supply = request.funding_target.checked_div(request.funding_token_price).unwrap();

            let pod_address = Self::env().account_id();
            let endowment = Self::env().balance() / 2;
            let caller = Self::env().caller();

            let pod_token =
                Erc20::new_optional(supply, Some(request.pod_token_name), Some(request.pod_token_symbol), Some(12))
                    .endowment(endowment)
                    .code_hash(request.erc20_code_hash)
                    .salt_bytes(pod_address.into_bytes())
                    .instantiate()
                    .expect("instantiate pod_token");

            let mut media_contract = request.media_contract;

            let media_ids: StorageVec<u64> = request
                .medias
                .into_iter()
                .map(|media| {
                    let media = media.into_media_request(caller, pod_address);
                    media_contract.create_media(media).expect("creating media")
                })
                .collect();

            Self {
                creator: caller,
                created_at: Self::env().block_timestamp(),
                spread: request.spread,
                funding_token: request.funding_token.clone(),
                funding_token_price: request.funding_token_price,
                funding_target: request.funding_target,
                funding_date: request.funding_date,
                max_price: request.max_price,
                max_supply: request.max_supply,
                amm_curve: request.amm,
                pod_token,
                media: media_contract,
                amm_address: pod_address,
                state: InvestingPodState {
                    status: InvestingPodStatus::Formation,
                    registered_media: 0,
                    total_media: media_ids.len(),
                    supply_released: 0,
                    raised_funds: 0,
                },
                media_ids,
            }
        }

        /// Sets the parameters of a media object. May only be called once, which then registers the
        /// media.
        #[ink(message)]
        pub fn register_media(&mut self, request: RegisterMediaRequest) -> Result<()> {
            let now = self.env().block_timestamp();
            if request.release_date < now {
                return Err(Error::ReleaseDateMustBeInFuture);
            }

            let caller = self.env().caller();

            // get the media by id
            let mut media = self.media.get_media(request.media_id).ok_or(Error::MediaNotFound)?;
            // check that the caller owns the media.
            if media.creator != caller {
                return Err(Error::Unauthorized);
            }

            if media.is_registered {
                return Err(Error::MediaAlreadyRegistered);
            }
            media.release_date = request.release_date;
            media.is_registered = true;
            media.view_conditions.viewing_type = request.payment_type;
            media.view_conditions.price = request.price;
            media.view_conditions.viewing_token = request.funding_token;

            self.media.update_collabs(media.id, request.collabs)?;
            self.media.update_media(media.into())?;
            self.state.increment_registered_media();
            Ok(())
        }

        /// Sets the media.is_uploaded field to true.
        ///
        /// # Restrictions
        ///
        /// * May only be called by the media creator.
        /// * Only registered media may be uploaded.
        #[ink(message)]
        pub fn upload_media(&mut self, media_id: MediaId) -> Result<()> {
            let mut media = self.media.get_media(media_id).ok_or(Error::MediaNotFound)?;

            if !media.is_registered {
                return Err(Error::MediaNotRegistered);
            }

            if media.creator != self.env().caller() {
                return Err(Error::Unauthorized);
            }

            media.is_uploaded = true;
            self.media.update_media(media.into())?;
            Ok(())
        }

        /// Purchases tokens from the pod for the funding price. Once the pods reaches the funding
        /// target, it will transition to trading state.
        #[ink(message)]
        pub fn invest_pod(&mut self, amount: Balance) -> Result<()> {
            let caller = self.env().caller();
            let contract_account_id = self.env().account_id();
            let now = self.env().block_timestamp();

            if self.funding_date < now {
                return Err(Error::FundingClosed);
            }

            if !self.state.status.is_investing() {
                return Err(Error::PodNotInInvestState);
            }

            // this should never panic since both funding_target and state.raised_funds are checked
            // before by the contract.
            let remaining = self.funding_target.checked_sub(self.state.raised_funds).unwrap();
            let amount = core::cmp::min(remaining, amount);
            let amount_pod_tokens = amount / self.funding_token_price;

            self.funding_token.transfer_from(caller, contract_account_id, amount)?;
            self.pod_token.transfer(caller, amount_pod_tokens)?;
            self.state.raised_funds += amount;

            if self.state.raised_funds >= self.funding_target {
                self.state.status = InvestingPodStatus::Trading
            }

            Ok(())
        }

        /// Buys pod tokens using the pod's AMM.
        #[ink(message)]
        pub fn buy_pod_tokens(&mut self, amount: Balance) -> Result<()> {
            let caller = self.env().caller();
            let contract_account_id = self.env().account_id();
            let amm = self.amm();

            // this will panic for ridiculous numbers (Balance::MAX for example). The caller should
            // never be passing in those numbers.
            let charged_amount = amm.buy(self.state.supply_released, amount).unwrap();

            // Balance should always be convertible to u128.
            self.funding_token.transfer_from(caller, contract_account_id, charged_amount)?;
            self.pod_token.mint(caller, amount)?;
            self.state.supply_released += amount;
            Ok(())
        }

        /// Sells pod tokens using the pod's AMM.
        #[ink(message)]
        pub fn sell_pod_tokens(&mut self, amount: Balance) -> Result<()> {
            let caller = self.env().caller();
            let amm = self.amm();

            // this will panic for ridiculous numbers (Balance::MAX for example). The caller should
            // never be passing in those numbers.
            let charged_amount = amm.sell(self.state.supply_released, amount).unwrap();
            self.pod_token.burn_from(caller, amount)?;

            // Balance should always be convertible to u128.
            self.funding_token.transfer(caller, charged_amount)?;
            self.state.supply_released = self.state.supply_released.checked_sub(amount).unwrap();
            Ok(())
        }

        /// AccountId of the pod creator.
        #[ink(message)]
        pub fn creator(&self) -> AccountId { self.creator }

        pub fn amm(&self) -> amm::Amm {
            // When pod parameters are set, check should be made so that the Amm is in a correct state.
            // later we will move to a dedicated AMM contract which ensures this even more.
            amm::Amm::new(self.amm_curve, self.funding_token_price, self.max_price, self.max_supply).unwrap()
        }
    }
}
