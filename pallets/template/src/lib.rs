//! # Template Pallet
//!
//! A pallet with validator registration, staking, nomination, and reward functionality.

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
	use frame_support::{pallet_prelude::*, traits::{Currency, ReservableCurrency, Get}};
	use frame_system::pallet_prelude::*;

	type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

	// Simple nominator info structure
	#[derive(Clone, Encode, Decode, Eq, PartialEq, RuntimeDebug, MaxEncodedLen, TypeInfo)]
	pub struct Nomination<AccountId, Balance> {
		pub validator: AccountId,
		pub amount: Balance,
	}

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
		/// Minimum amount required to nominate
		#[pallet::constant]
		type MinNomination: Get<BalanceOf<Self>>;
		/// Maximum number of nominations per nominator
		#[pallet::constant]
		type MaxNominations: Get<u32>;
		/// Reward rate per block (as a percentage of total staked)
		#[pallet::constant]
		type RewardRate: Get<u32>;
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

	// Storage for nominators and their nominations
	#[pallet::storage]
	#[pallet::getter(fn nominators)]
	pub type Nominators<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BoundedVec<Nomination<T::AccountId, BalanceOf<T>>, T::MaxNominations>,
		ValueQuery
	>;

	// Total stake for each validator (self + nominations)
	#[pallet::storage]
	#[pallet::getter(fn total_validator_stake)]
	pub type TotalValidatorStake<T: Config> = StorageMap<
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

	// Pending rewards for each account
	#[pallet::storage]
	#[pallet::getter(fn pending_rewards)]
	pub type PendingRewards<T: Config> = StorageMap<
		_,
		Blake2_128Concat,
		T::AccountId,
		BalanceOf<T>,
		ValueQuery
	>;

	// Last block rewards were distributed
	#[pallet::storage]
	#[pallet::getter(fn last_reward_block)]
	pub type LastRewardBlock<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// A new validator has been registered with stake [validator, stake]
		ValidatorRegistered(T::AccountId, BalanceOf<T>),
		/// A validator has been removed and stake returned [validator, stake]
		ValidatorRemoved(T::AccountId, BalanceOf<T>),
		/// A nomination has been made [nominator, validator, amount]
		Nomination(T::AccountId, T::AccountId, BalanceOf<T>),
		/// A nomination has been withdrawn [nominator, validator, amount]
		NominationWithdrawn(T::AccountId, T::AccountId, BalanceOf<T>),
		/// Rewards have been claimed [account, amount]
		RewardsClaimed(T::AccountId, BalanceOf<T>),
		/// Rewards have been distributed [block_number, total_rewards]
		RewardsDistributed(BlockNumberFor<T>, BalanceOf<T>),
	}

	#[pallet::error]
	pub enum Error<T> {
		/// Already registered as validator
		AlreadyValidator,
		/// Not a validator
		NotValidator,
		/// Stake is below minimum required
		StakeBelowMinimum,
		/// Nomination is below minimum required
		NominationBelowMinimum,
		/// Insufficient balance
		InsufficientBalance,
		/// Maximum nominations reached
		MaxNominationsReached,
		/// Already nominated this validator
		AlreadyNominated,
		/// Nomination not found
		NominationNotFound,
		/// No rewards to claim
		NoRewards,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: BlockNumberFor<T>) -> Weight {
			// Distribute rewards every block
			Self::distribute_rewards(n);
			Weight::zero()
		}
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
			
			// Initialize total validator stake (self stake + nominations)
			TotalValidatorStake::<T>::insert(&who, stake);
			
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
			TotalValidatorStake::<T>::remove(&who);
			
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

		/// Nominate a validator with the specified amount
		#[pallet::call_index(2)]
		#[pallet::weight(T::WeightInfo::do_something())]
		pub fn nominate(
			origin: OriginFor<T>,
			validator: T::AccountId,
			#[pallet::compact] amount: BalanceOf<T>
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			
			// Check if validator exists
			ensure!(Validators::<T>::contains_key(&validator), Error::<T>::NotValidator);
			
			// Check minimum nomination
			ensure!(amount >= T::MinNomination::get(), Error::<T>::NominationBelowMinimum);
			
			// Check balance
			ensure!(T::Currency::free_balance(&who) >= amount, Error::<T>::InsufficientBalance);
			
			// Check if already nominated this validator
			let mut nominations = Nominators::<T>::get(&who);
			ensure!(!nominations.iter().any(|n| n.validator == validator), Error::<T>::AlreadyNominated);
			
			// Check max nominations
			ensure!(nominations.len() < T::MaxNominations::get() as usize, Error::<T>::MaxNominationsReached);
			
			// Reserve the amount
			T::Currency::reserve(&who, amount)?;
			
			// Add nomination
			let nomination = Nomination {
				validator: validator.clone(),
				amount,
			};
			nominations.try_push(nomination).map_err(|_| Error::<T>::MaxNominationsReached)?;
			Nominators::<T>::insert(&who, nominations);
			
			// Update total validator stake
			TotalValidatorStake::<T>::mutate(&validator, |total| {
				*total = total.checked_add(&amount).unwrap_or(*total);
			});
			
			// Update total staked
			let old_total = TotalStaked::<T>::get();
			let new_total = old_total.checked_add(&amount).unwrap_or(old_total);
			TotalStaked::<T>::put(new_total);
			
			// Emit event
			Self::deposit_event(Event::Nomination(who, validator, amount));
			
			Ok(())
		}

		/// Withdraw a nomination from a validator
		#[pallet::call_index(3)]
		#[pallet::weight(T::WeightInfo::cause_error())]
		pub fn withdraw_nomination(
			origin: OriginFor<T>,
			validator: T::AccountId
		) -> DispatchResult {
			let who = ensure_signed(origin)?;
			
			// Get nominations
			let mut nominations = Nominators::<T>::get(&who);
			
			// Find the nomination
			let position = nominations.iter().position(|n| n.validator == validator)
				.ok_or(Error::<T>::NominationNotFound)?;
			
			// Get the nomination amount
			let amount = nominations[position].amount;
			
			// Remove the nomination
			nominations.swap_remove(position);
			Nominators::<T>::insert(&who, nominations);
			
			// Unreserve the amount
			T::Currency::unreserve(&who, amount);
			
			// Update total validator stake
			TotalValidatorStake::<T>::mutate(&validator, |total| {
				*total = total.checked_sub(&amount).unwrap_or(*total);
			});
			
			// Update total staked
			let old_total = TotalStaked::<T>::get();
			let new_total = old_total.checked_sub(&amount).unwrap_or(old_total);
			TotalStaked::<T>::put(new_total);
			
			// Emit event
			Self::deposit_event(Event::NominationWithdrawn(who, validator, amount));
			
			Ok(())
		}

		/// Claim pending rewards
		#[pallet::call_index(4)]
		#[pallet::weight(T::WeightInfo::do_something())]
		pub fn claim_rewards(origin: OriginFor<T>) -> DispatchResult {
			let who = ensure_signed(origin)?;
			
			// Get pending rewards
			let rewards = PendingRewards::<T>::get(&who);
			ensure!(!rewards.is_zero(), Error::<T>::NoRewards);
			
			// Clear pending rewards
			PendingRewards::<T>::remove(&who);
			
			// Transfer rewards
			T::Currency::deposit_creating(&who, rewards);
			
			// Emit event
			Self::deposit_event(Event::RewardsClaimed(who, rewards));
			
			Ok(())
		}
	}

	impl<T: Config> Pallet<T> {
		// Constants for reward calculation (using integer math)
		const BLOCKS_PER_YEAR: u32 = 5_256_000; // Assuming 6-second blocks
		const VALIDATOR_INFLATION_RATE_NUMERATOR: u32 = 15; // 15%
		const VALIDATOR_INFLATION_RATE_DENOMINATOR: u32 = 100;
		const NOMINATOR_INFLATION_RATE_NUMERATOR: u32 = 10; // 10%
		const NOMINATOR_INFLATION_RATE_DENOMINATOR: u32 = 100;

		// Distribute rewards to validators and nominators
		fn distribute_rewards(block_number: BlockNumberFor<T>) {
			// Get last reward block
			let last_block = LastRewardBlock::<T>::get();
			
			// Skip if rewards were already distributed in this block
			if last_block == block_number {
				return;
			}
			
			// Update last reward block
			LastRewardBlock::<T>::put(block_number);
			
			// Get total staked
			let total_staked = TotalStaked::<T>::get();
			if total_staked.is_zero() {
				return;
			}
			
			// Calculate total rewards (reward_rate % of total staked)
			// Use a simple approach to avoid type conversion issues
			let reward_rate = T::RewardRate::get();
			if reward_rate == 0 {
				return;
			}
			
			// Create a small reward for each validator based on their stake
			for (validator, validator_stake) in Validators::<T>::iter() {
				// Get total stake for this validator
				let total_validator_stake = TotalValidatorStake::<T>::get(&validator);
				if total_validator_stake.is_zero() {
					continue;
				}
				
				// Create a small reward for the validator (15% APY)
				let validator_reward = validator_stake
					.checked_mul(&Self::VALIDATOR_INFLATION_RATE_NUMERATOR.into())
					.and_then(|r| r.checked_div(&(Self::BLOCKS_PER_YEAR * Self::VALIDATOR_INFLATION_RATE_DENOMINATOR).into()))
					.unwrap_or_else(Zero::zero);
				if !validator_reward.is_zero() {
					// Add to validator's pending rewards
					PendingRewards::<T>::mutate(&validator, |rewards| {
						*rewards = rewards.checked_add(&validator_reward).unwrap_or(*rewards);
					});
					
					// Create the reward tokens
					let _ = T::Currency::deposit_creating(&validator, validator_reward);
				}
				
				// Process nominators for this validator
				for (nominator, nominations) in Nominators::<T>::iter() {
					for nomination in nominations.iter() {
						if nomination.validator != validator {
							continue;
						}
						
						// Create a small reward for the nominator (10% APY)
						let nominator_reward = nomination.amount
							.checked_mul(&Self::NOMINATOR_INFLATION_RATE_NUMERATOR.into())
							.and_then(|r| r.checked_div(&(Self::BLOCKS_PER_YEAR * Self::NOMINATOR_INFLATION_RATE_DENOMINATOR).into()))
							.unwrap_or_else(Zero::zero);
						if !nominator_reward.is_zero() {
							// Add to nominator's pending rewards
							PendingRewards::<T>::mutate(&nominator, |rewards| {
								*rewards = rewards.checked_add(&nominator_reward).unwrap_or(*rewards);
							});
							
							// Create the reward tokens
							let _ = T::Currency::deposit_creating(&nominator, nominator_reward);
						}
					}
				}
			}
			
			// Emit event with a simple reward amount
			let total_reward = total_staked
				.checked_mul(&Self::VALIDATOR_INFLATION_RATE_NUMERATOR.into())
				.and_then(|r| r.checked_div(&(Self::BLOCKS_PER_YEAR * Self::VALIDATOR_INFLATION_RATE_DENOMINATOR).into()))
				.unwrap_or_else(Zero::zero);
			Self::deposit_event(Event::RewardsDistributed(block_number, total_reward));
		}
	}
}