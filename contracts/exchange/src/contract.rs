//! Contains the code for the contract

mod constants {
    /// Salt for generating exchange id
    pub const EXCHANGE_ID_SALT: [u8; 4] = *b"exid";
    /// Salt for generating offer id
    pub const OFFER_ID_SALT: [u8; 4] = *b"ofid";
}

/// Contains fhr contract code
#[ink_lang::contract]
mod contract {
    use super::*;
    use crate::{model::*, Error, Result};
    use ink_prelude::{vec, vec::Vec};

    // ============= Events

    /// Emitted when an exchange is created
    #[ink(event)]
    #[derive(derive_new::new)]
    pub struct CreatedExchange {
        /// Ouput of the event
        pub output: event_output::CreatedExchangeOutput,
    }

    /// An offer was placed
    #[ink(event)]
    #[derive(derive_new::new)]
    pub struct PlacedOffer {
        /// The output of the event
        pub output: event_output::PlacedOfferOutput,
    }

    /// An offer was canceled
    #[ink(event)]
    #[derive(derive_new::new)]
    pub struct CanceledOffer {
        /// The output of the event
        pub output: event_output::CanceledOfferOutput,
    }

    // ======== Storage

    /// Storage for the social token
    #[ink(storage)]
    pub struct Exchanges {
        /// The exchanges by id
        exchanges_by_id: ink_storage::collections::HashMap<ExchangeId, Exchange>,
        /// The offer ids by exchange id (allows getting all offers for an exchange id)
        offer_ids_by_exchange_id: ink_storage::collections::HashMap<ExchangeId, Vec<OfferId>>,
        /// The offers by id
        offers_by_id: ink_storage::collections::HashMap<OfferId, Offer>,
        /// Nonce used for random seed
        nonce: ink_storage::lazy::Lazy<u128>,
    }

    impl Exchanges {
        /// Create a new instance.
        /// * `token_accounts_id` - The `AccountId` for the `TokenAccounts` contract`
        #[allow(clippy::new_without_default)]
        #[ink(constructor)]
        pub fn new() -> Self {
            Self {
                exchanges_by_id: Default::default(),
                offer_ids_by_exchange_id: Default::default(),
                offers_by_id: Default::default(),
                nonce: Default::default(),
            }
        }

        /// Creates a new exchange for selling a given asset. It could be an NFT, a social token...
        ///
        /// * `exchange_token_id` - ExchangeToken token that is going to be traded through this order book model
        /// * `initial_amount` - InitialAmount of exchangeToken to sell
        /// * `offer_token_id` - Token of this first selling offer
        /// * `price` - Price per each exchange token of the Initial supply
        #[ink(message)]
        pub fn create_exchange(&mut self, input: CreateExchangeRequest) -> Result<()> {
            // get tokens
            let mut exchange_token = input.exchange_token.into();
            let offer_token = input.offer_token.into();

            // create exchange
            let exchange_id = self.random_hash(constants::EXCHANGE_ID_SALT);
            let caller = self.env().caller();
            // let exchange_address = self.random_hash(EXCHANGE_ADDRESS_SALT).to_account_id();
            let exchange = Exchange {
                id: exchange_id,
                creator: caller,
                exchange_token,
                initial_amount: input.initial_amount,
                price: input.price,
            };

            // create offer
            let offer_id = self.random_hash(constants::OFFER_ID_SALT);
            let offer = Offer {
                id: offer_id,
                exchange_id: exchange.id,
                offer_type: OfferType::Sell,
                creator: caller,
                price: input.price,
                amount: input.initial_amount,
                token: offer_token,
            };

            // transfer funds to the exchange
            exchange_token.transfer_from(offer.creator, self.env().account_id(), offer.amount)?;

            // store the data
            self.exchanges_by_id.insert(exchange_id, exchange);
            self.offers_by_id.insert(offer_id, offer);
            self.offer_ids_by_exchange_id.entry(exchange_id).and_modify(|x| x.push(offer_id)).or_insert(vec![offer_id]);

            // emit event
            self.env().emit_event(CreatedExchange::new(event_output::CreatedExchangeOutput { exchange_id, offer_id }));

            Ok(())
        }

        /// Get an exchange by id
        #[ink(message)]
        pub fn get_exchange_by_id(&self, id: ExchangeId) -> Option<ExchangeInfo> {
            self.exchanges_by_id.get(&id).map(|x| ExchangeInfo {
                id,
                creator_address: x.creator,
                exchange_token: x.exchange_token.into(),
                initial_amount: x.initial_amount,
                price: x.price,
            })
        }

        /// Get all of the offers for an exchange
        #[ink(message)]
        pub fn get_exchange_offers(&self, id: ExchangeId) -> Option<Vec<OfferInfo>> {
            self.offer_ids_by_exchange_id.get(&id).map(|x| {
                x.iter()
                    .flat_map(|y| {
                        self.offers_by_id.get(y).map(|o| OfferInfo {
                            id: o.id,
                            exchange_id: o.exchange_id,
                            r#type: o.offer_type,
                            creator_address: o.creator,
                            price: o.price,
                            amount: o.amount,
                            offer_token: o.token.into(),
                        })
                    })
                    .collect()
            })
        }

        /// Creates a new buy offer for the asset. It could be an NFT, a social token...
        ///
        /// * `exchange_id` - Id of the exchange
        /// * `token_id` - Token of the offer
        /// * `amount` - Amount of token for the order book
        /// * `price` - Price per each exchange token of the Initial supply
        #[ink(message)]
        pub fn place_buying_offer(&mut self, input: PlaceOfferRequest) -> Result<()> {
            self.place_offer(input, OfferType::Buy)
        }

        /// Creates a new sell offer for the asset. It could be an NFT, a social token...
        ///
        /// * `exchange_id` - Id of the exchange
        /// * `token_id` - Token of the offer
        /// * `amount` - Amount of token for the order book
        /// * `price` - Price per each exchange token of the Initial supply
        #[ink(message)]
        pub fn place_selling_offer(&mut self, input: PlaceOfferRequest) -> Result<()> {
            self.place_offer(input, OfferType::Sell)
        }

        fn place_offer(&mut self, input: PlaceOfferRequest, offer_type: OfferType) -> Result<()> {
            // create the offer
            let offer_id = self.random_hash(constants::OFFER_ID_SALT);
            let mut offer_token = input.offer_token.into();
            let offer = Offer {
                id: offer_id,
                exchange_id: input.exchange_id,
                offer_type,
                creator: self.env().caller(),
                price: input.price,
                amount: input.amount,
                token: offer_token,
            };

            // transfer the tokens
            let contract_account_id = self.env().account_id();
            let exchange = self.get_exchange_mut(&input.exchange_id)?;
            match offer_type {
                OfferType::Buy => {
                    offer_token.transfer_from(offer.creator, contract_account_id, Some(offer.price * offer.amount))?
                }
                OfferType::Sell => {
                    exchange.exchange_token.transfer_from(offer.creator, contract_account_id, Some(offer.amount))?
                }
            }

            // store the data
            self.offers_by_id.insert(offer.id, offer);
            self.offer_ids_by_exchange_id
                .entry(input.exchange_id)
                .and_modify(|x| x.push(offer_id))
                .or_insert(vec![offer_id]);

            self.env().emit_event(PlacedOffer::new(event_output::PlacedOfferOutput { offer_id }));
            Ok(())
        }

        /// Cancels a buying offer
        ///
        /// * `exchange_id` - The exchange id
        /// * `offer_id` - The offer id
        #[ink(message)]
        pub fn cancel_buying_offer(&mut self, input: CancelOfferRequest) -> Result<()> {
            self.cancel_offer(input.exchange_id, input.offer_id, OfferType::Buy)
        }

        /// Cancels a selling offer
        ///
        /// * `exchange_id` - The exchange id
        /// * `offer_id` - The offer id
        #[ink(message)]
        pub fn cancel_selling_offer(&mut self, input: CancelOfferRequest) -> Result<()> {
            self.cancel_offer(input.exchange_id, input.offer_id, OfferType::Sell)
        }

        /// Cancels an offer
        fn cancel_offer(&mut self, exchange_id: ExchangeId, offer_id: OfferId, offer_type: OfferType) -> Result<()> {
            let exchange = self.get_exchange(&exchange_id)?;
            let offer = self.get_offer(&offer_id)?;
            if offer.offer_type != offer_type {
                return Err(Error::OfferTypeMismatch);
            }

            // transfer the tokens back to the creator
            match offer.offer_type {
                OfferType::Buy => offer.token.clone().transfer(offer.creator, Some(offer.amount * offer.price))?,
                OfferType::Sell => exchange.exchange_token.clone().transfer(offer.creator, Some(offer.amount))?,
            }

            // update storage
            self.offers_by_id.take(&offer_id);
            if let Some(offer_ids) = self.offer_ids_by_exchange_id.get_mut(&exchange_id) {
                if let Some((index, _)) = offer_ids.iter().enumerate().find(|(_, x)| **x == offer_id) {
                    offer_ids.remove(index);
                }
                if offer_ids.is_empty() {
                    self.offer_ids_by_exchange_id.take(&exchange_id);
                }
            }

            // emit event
            self.env().emit_event(CanceledOffer::new(event_output::CanceledOfferOutput { offer_id }));
            Ok(())
        }

        /// Buy from a sell offer of a given asset.
        ///
        /// `exchange_id` - Id of the exchange
        /// `offer_id` - Id of the offer
        /// `buyer_account` - Account of the buyer
        /// `amount` - Amount of token for the order book
        #[ink(message)]
        pub fn buy_from_offer(&mut self, input: OfferRequest) -> Result<()> {
            let mut exchange_token = self.get_exchange(&input.exchange_id).map(|x| x.exchange_token)?;
            let offer = self.get_offer_mut(&input.offer_id)?;
            if offer.offer_type != OfferType::Sell {
                return Err(Error::OfferTypeMismatch);
            }

            if input.amount > offer.amount {
                return Err(Error::InsufficientBalance);
            }

            // transfer offer tokens from buyer to the offer's creator
            offer.token.transfer_from(input.address, offer.creator, Some(offer.price * input.amount))?;
            // transfer exchange token from exchange to the buyer
            exchange_token.transfer(input.address, input.amount)?;

            // update offer state
            offer.amount -= input.amount;
            Ok(())
        }

        /// Sell to a buy offer
        ///
        /// `exchange_id` - Id of the exchange
        /// `offer_id` - Id of the offer
        /// `seller_account` - Account of the buyer
        /// `amount` - Amount of token for the order book
        #[ink(message)]
        pub fn sell_from_offer(&mut self, input: OfferRequest) -> Result<()> {
            let mut exchange_token = self.get_exchange(&input.exchange_id).map(|x| x.exchange_token)?;
            let offer = self.get_offer_mut(&input.offer_id)?;
            if offer.offer_type != OfferType::Buy {
                return Err(Error::OfferTypeMismatch);
            }

            if input.amount > offer.amount {
                return Err(Error::InsufficientBalance);
            }

            // transfer exchange tokens from the exchange to the offer creator
            exchange_token.transfer(offer.creator, Some(input.amount))?;
            // transfer offer tokens from the exchange to the seller
            offer.token.transfer(input.address, Some(offer.price * input.amount))?;

            // update offer state
            offer.amount -= input.amount;
            Ok(())
        }

        /// Generate a random `Hash` based on caller, nonce, and salt
        fn random_hash(&mut self, salt: [u8; 4]) -> Hash {
            use contract_utils::AccountIdExt;

            let mut subject = [0_u8; 32];
            subject[0..12].copy_from_slice(&self.env().caller().into_bytes()[..12]);
            subject[12..16].copy_from_slice(&salt);
            subject[16..32].copy_from_slice(&self.increment_nonce().to_le_bytes());
            self.env().random(&subject)
        }
    }

    impl Exchanges {
        /// Get an exchange
        fn get_exchange(&self, exchange_id: &ExchangeId) -> Result<&Exchange> {
            self.exchanges_by_id.get(exchange_id).ok_or(Error::ExchangeNotFound)
        }

        /// Get a mutable exchange
        fn get_exchange_mut(&mut self, exchange_id: &ExchangeId) -> Result<&mut Exchange> {
            self.exchanges_by_id.get_mut(exchange_id).ok_or(Error::ExchangeNotFound)
        }

        /// Get an offer
        fn get_offer(&self, offer_id: &OfferId) -> Result<&Offer> {
            self.offers_by_id.get(offer_id).ok_or(Error::OfferNotFound)
        }

        /// Get a mutable offer
        fn get_offer_mut(&mut self, offer_id: &OfferId) -> Result<&mut Offer> {
            self.offers_by_id.get_mut(offer_id).ok_or(Error::OfferNotFound)
        }

        /// Increments nonce and returns current
        fn increment_nonce(&mut self) -> u128 {
            let current = *self.nonce;
            *self.nonce += 1;
            current
        }
    }
}
