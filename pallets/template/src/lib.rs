//! # Template Pallet
//!
//! A pallet with validator registration and staking functionality.

#![cfg_attr(not(feature = "std"), no_std)]

pub use pallet::*;

#[cfg(test)]
mod mock;
#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{pallet_prelude::*, traits::{Currency, ReservableCurrency}};
	use frame_system::pallet_prelude::*;

	type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// The overarching runtime event type.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// The currency used for staking
		type Currency: Currency<Self::AccountId> + ReservableCurrency<Self::AccountId>;
		/// Minimum amount required to stake as a validator
		#[pallet::constant]
		type MinStake: Get<BalanceOf<Self>>;
		/// A type representing the weights required by the dispatchables of this pallet.
		type WeightInfo: WeightInfo;
	}

	// Storage for validators and their stake
	#[pallet::storage]
	#[pallet::getter(fn validators)]
	pub type Validators<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BalanceOf<T>,
		ValueQuery
	>;

	// Total number of validators
	#[pallet::storage]
	#[pallet::getter(fn validator_count)]
	pub type ValidatorCount<T: Config> = StorageValue<_, u32, ValueQuery>;

	// Total staked amount
	#[pallet::storage]
	#[pallet::getter(fn total_staked)]
	pub type TotalStaked<T: Config> = StorageValue<_, BalanceOf<T>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new validator has been registered with stake [validator, stake]
		ValidatorRegistered(T::AccountId, BalanceOf<T>),
		/// A validator has been removed and stake returned [validator, stake]
		ValidatorRemoved(T::AccountId, BalanceOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Already registered as validator
		AlreadyValidator,
		/// Not a validator
		NotValidator,
		/// Stake is below minimum required
		StakeBelowMinimum,
		/// Insufficient balance
		InsufficientBalance,
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {
		/// Register as a validator with the specified stake
		#[pallet::call_index(0)]
		#[pallet::weight(T::WeightInfo::do_something())]
		pub fn register_validator(
			origin: OriginFor<T>,
			#[pallet::compact] stake: BalanceOf<T>
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			
			// Check if already a validator
			ensure!(!Validators::<T>::contains_key(&who), Error::<T>::AlreadyValidator);
			
			// Check minimum stake
			ensure!(stake >= T::MinStake::get(), Error::<T>::StakeBelowMinimum);
			
			// Check balance
			ensure!(T::Currency::free_balance(&who) >= stake, Error::<T>::InsufficientBalance);
			
			// Reserve the stake
			T::Currency::reserve(&who, stake)?;
			
			// Register as validator with stake
			Validators::<T>::insert(&who, stake);
			
			// Increment validator count
			ValidatorCount::<T>::mutate(|count| *count += 1);
			
			// Update total staked
			let old_total = TotalStaked::<T>::get();
			let new_total = old_total.checked_add(&stake).unwrap_or(old_total);
			TotalStaked::<T>::put(new_total);
			
			// Emit event
			Self::deposit_event(Event::ValidatorRegistered(who, stake));
			
			Ok(())
		}
		
		/// Remove validator status and return stake
		#[pallet::call_index(1)]
		#[pallet::weight(T::WeightInfo::cause_error())]
		pub fn remove_validator(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			
			// Check if is a validator and get stake
			let stake = Validators::<T>::get(&who);
			ensure!(stake > BalanceOf::<T>::zero(), Error::<T>::NotValidator);
			
			// Unreserve the stake
			T::Currency::unreserve(&who, stake);
			
			// Remove validator
			Validators::<T>::remove(&who);
			
			// Decrement validator count
			ValidatorCount::<T>::mutate(|count| *count = count.saturating_sub(1));
			
			// Update total staked
			let old_total = TotalStaked::<T>::get();
			let new_total = old_total.checked_sub(&stake).unwrap_or(old_total);
			TotalStaked::<T>::put(new_total);
			
			// Emit event
			Self::deposit_event(Event::ValidatorRemoved(who, stake));
			
			Ok(())
		}
	}
}