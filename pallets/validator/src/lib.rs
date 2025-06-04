#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
    }

    // Simple storage for validators
    #[pallet::storage]
    #[pallet::getter(fn validators)]
    pub type Validators<T: Config> = StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

    // Total number of validators
    #[pallet::storage]
    #[pallet::getter(fn validator_count)]
    pub type ValidatorCount<T: Config> = StorageValue<_, u32, ValueQuery>;

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        // New validator registered
        ValidatorRegistered(T::AccountId),
        // Validator removed
        ValidatorRemoved(T::AccountId),
    }

    #[pallet::error]
    pub enum Error<T> {
        // Already registered as validator
        AlreadyValidator,
        // Not a validator
        NotValidator,
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        // Register as a validator
        #[pallet::call_index(0)]
        #[pallet::weight(10_000)]
        pub fn register_validator(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Check if already a validator
            ensure!(!Validators::<T>::contains_key(&who), Error::<T>::AlreadyValidator);
            
            // Register as validator
            Validators::<T>::insert(&who, true);
            
            // Increment validator count
            ValidatorCount::<T>::mutate(|count| *count += 1);
            
            // Emit event
            Self::deposit_event(Event::ValidatorRegistered(who));
            
            Ok(())
        }
        
        // Remove validator status
        #[pallet::call_index(1)]
        #[pallet::weight(10_000)]
        pub fn remove_validator(origin: OriginFor<T>) -> DispatchResult {
            let who = ensure_signed(origin)?;
            
            // Check if is a validator
            ensure!(Validators::<T>::contains_key(&who), Error::<T>::NotValidator);
            
            // Remove validator
            Validators::<T>::remove(&who);
            
            // Decrement validator count
            ValidatorCount::<T>::mutate(|count| *count = count.saturating_sub(1));
            
            // Emit event
            Self::deposit_event(Event::ValidatorRemoved(who));
            
            Ok(())
        }
    }
}