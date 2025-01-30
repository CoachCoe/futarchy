// Futarchy Pallet - Initial Prototype

#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{
    decl_module, 
    decl_storage, 
    decl_event, 
    decl_error, 
    ensure,
    dispatch::DispatchResult,
    traits::{Get, Currency, ReservableCurrency}
};
use frame_system::{
    self as system, 
    ensure_signed
};
use sp_runtime::{
    traits::{Hash, Zero, CheckedAdd, CheckedSub},
    RuntimeDebug
};
use sp_std::prelude::*;

// Market Types
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum MarketType {
    Binary,
    Scalar,
    Categorical
}

// Market Status
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub enum MarketStatus {
    Created,
    Active,
    Resolved,
    Cancelled
}

// Prediction Market Structure
#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug)]
pub struct PredictionMarket<AccountId, Balance, BlockNumber> {
    id: Hash,
    creator: AccountId,
    market_type: MarketType,
    status: MarketStatus,
    total_liquidity: Balance,
    creation_block: BlockNumber,
    resolution_block: Option<BlockNumber>,
}

// Pallet Configuration Trait
pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    type Currency: ReservableCurrency<Self::AccountId>;
    type MarketCreationDeposit: Get<BalanceOf<Self>>;
}

// Pallet Declaration
decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;
        fn deposit_event() = default;

        // Create a new prediction market
        #[weight = 10_000]
        pub fn create_market(
            origin, 
            market_type: MarketType
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Ensure minimum deposit is paid
            let deposit = T::MarketCreationDeposit::get();
            T::Currency::reserve(&who, deposit)?;

            // Generate unique market ID
            let market_id = (system::Module::<T>::block_number(), who.clone(), market_type.clone()).using_encoded(T::Hashing::hash);

            // Create market
            let market = PredictionMarket {
                id: market_id,
                creator: who.clone(),
                market_type,
                status: MarketStatus::Created,
                total_liquidity: Zero::zero(),
                creation_block: system::Module::<T>::block_number(),
                resolution_block: None,
            };

            // Store market
            Markets::<T>::insert(market_id, market);

            // Emit event
            Self::deposit_event(RawEvent::MarketCreated(who, market_id));

            Ok(())
        }

        // Resolve a prediction market
        #[weight = 10_000]
        pub fn resolve_market(
            origin, 
            market_id: T::Hash
        ) -> DispatchResult {
            let who = ensure_signed(origin)?;

            // Retrieve market
            let mut market = Markets::<T>::get(market_id)
                .ok_or(Error::<T>::MarketDoesNotExist)?;

            // Validate market can be resolved
            ensure!(market.status == MarketStatus::Active, Error::<T>::MarketNotResolvable);

            // Update market status
            market.status = MarketStatus::Resolved;
            market.resolution_block = Some(system::Module::<T>::block_number());

            // Store updated market
            Markets::<T>::insert(market_id, market);

            // Emit event
            Self::deposit_event(RawEvent::MarketResolved(who, market_id));

            Ok(())
        }
    }
}

// Storage Declarations
decl_storage! {
    trait Store for Module<T: Config> as FutarchyMarkets {
        // Store all markets
        Markets get(fn markets): map hasher(blake2_128_concat) T::Hash => Option<PredictionMarket<T::AccountId, BalanceOf<T>, T::BlockNumber>>;
        
        // Total number of markets
        MarketCount get(fn market_count): u64;
    }
}

// Event Declarations
decl_event!(
    pub enum Event<T> where AccountId = <T as frame_system::Config>::AccountId {
        // Market created with its unique ID
        MarketCreated(AccountId, T::Hash),
        // Market resolved
        MarketResolved(AccountId, T::Hash),
    }
}

// Error Declarations
decl_error! {
    pub enum Error for Module<T: Config> {
        // Market does not exist
        MarketDoesNotExist,
        // Market is not in a state that can be resolved
        MarketNotResolvable,
        // Insufficient funds for market creation
        InsufficientFunds,
    }
}

// Module Implementation
impl<T: Config> Module<T> {
    // Helper function to get total market count
    pub fn market_count() -> u64 {
        MarketCount::get()
    }
}
