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

	// Define EraIndex type
	pub type EraIndex = u32;

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

	// Current era index
	#[pallet::storage]
	#[pallet::getter(fn current_era)]
	pub type CurrentEra<T: Config> = StorageValue<_, EraIndex, ValueQuery>;

	// Start block of the current era
	#[pallet::storage]
	#[pallet::getter(fn era_start_block)]
	pub type EraStartBlock<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

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
		RewardsDistributed(EraIndex, BalanceOf<T>),
		/// A validator has been slashed [validator, amount, percentage]
		ValidatorSlashed(T::AccountId, BalanceOf<T>, u32),
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
		/// Invalid slash percentage
		InvalidSlashPercentage,
		/// Slash amount is zero
		ZeroSlashAmount,
		/// Insufficient stake for slashing
		InsufficientStake,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		fn on_initialize(n: BlockNumberFor<T>) -> Weight {
			// Check if it's time for a new era
			let era_duration = BlockNumberFor::<T>::from(Self::ERA_DURATION);
			let current_era = Self::current_era();
			let era_start_block = Self::era_start_block();
			
			if n >= era_start_block + era_duration {
				// Start a new era
				CurrentEra::<T>::put(current_era + 1);
				EraStartBlock::<T>::put(n);
				
				// Distribute rewards for the previous era
				Self::distribute_rewards(current_era);
				
				Weight::from_parts(10_000_000, 0)
			} else {
				Weight::from_parts(1_000_000, 0)
			}
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

		/// Slash a validator for misbehavior
		#[pallet::call_index(5)]
		#[pallet::weight(T::WeightInfo::do_something())]
		pub fn slash_validator(
			origin: OriginFor<T>,
			validator: T::AccountId,
			#[pallet::compact] slash_percent: u32,
		) -> DispatchResult {
			// Only allow governance or admin to slash
			ensure_root(origin)?;
			
			// Ensure slash percent is valid (1-100%)
			ensure!(slash_percent > 0 && slash_percent <= 100, Error::<T>::InvalidSlashPercentage);
			
			// Check if account is a validator
			let validator_stake = Validators::<T>::get(&validator);
			ensure!(validator_stake > BalanceOf::<T>::zero(), Error::<T>::NotValidator);
			
			// Calculate slash amount
			let slash_amount = validator_stake
				.checked_mul(&slash_percent.into())
				.and_then(|r| r.checked_div(&100u32.into()))
				.unwrap_or_else(Zero::zero);
			
			// Ensure slash amount is not zero
			ensure!(!slash_amount.is_zero(), Error::<T>::ZeroSlashAmount);
			
			// Slash the validator's stake
			Self::do_slash(&validator, slash_amount)?;
			
			// Emit event
			Self::deposit_event(Event::ValidatorSlashed(validator, slash_amount, slash_percent));
			
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

		// Era duration in blocks
    	const ERA_DURATION: u32 = 14_400; // 1 day with 6-second blocks
		
		// Distribute rewards to validators and nominators
		fn distribute_rewards(era: EraIndex) {
			// Get total staked
			let total_staked = TotalStaked::<T>::get();
			if total_staked.is_zero() {
				return;
			}
			
			// Calculate total rewards (reward_rate % of total staked)
			let reward_rate = T::RewardRate::get();
			if reward_rate == 0 {
				return;
			}
			
			// Create rewards for validators and nominators
			for (validator, validator_stake) in Validators::<T>::iter() {
				// Get total stake for this validator
				let total_validator_stake = TotalValidatorStake::<T>::get(&validator);
				if total_validator_stake.is_zero() {
					continue;
				}
				
				// Calculate validator's reward
				let validator_reward = validator_stake
					.checked_mul(&Self::VALIDATOR_INFLATION_RATE_NUMERATOR.into())
					.and_then(|r| r.checked_div(&(Self::BLOCKS_PER_YEAR * Self::VALIDATOR_INFLATION_RATE_DENOMINATOR).into()))
					.unwrap_or_else(Zero::zero)
					.checked_mul(&Self::ERA_DURATION.into())
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
						
						// Calculate nominator's reward
						let nominator_reward = nomination.amount
							.checked_mul(&Self::NOMINATOR_INFLATION_RATE_NUMERATOR.into())
							.and_then(|r| r.checked_div(&(Self::BLOCKS_PER_YEAR * Self::NOMINATOR_INFLATION_RATE_DENOMINATOR).into()))
							.unwrap_or_else(Zero::zero)
							.checked_mul(&Self::ERA_DURATION.into())
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
			
			// Emit event with total rewards for the era
			let total_reward = total_staked
				.checked_mul(&Self::VALIDATOR_INFLATION_RATE_NUMERATOR.into())
				.and_then(|r| r.checked_div(&(Self::BLOCKS_PER_YEAR * Self::VALIDATOR_INFLATION_RATE_DENOMINATOR).into()))
				.unwrap_or_else(Zero::zero)
				.checked_mul(&Self::ERA_DURATION.into())
				.unwrap_or_else(Zero::zero);
			
			Self::deposit_event(Event::RewardsDistributed(era, total_reward));
    	}

		// Helper function to slash a validator
		fn do_slash(validator: &T::AccountId, slash_amount: BalanceOf<T>) -> DispatchResult {
			ensure!(!slash_amount.is_zero(), Error::<T>::ZeroSlashAmount);

			let current_stake = Validators::<T>::get(validator);
			let remaining_stake = current_stake.checked_sub(&slash_amount)
				.ok_or(Error::<T>::InsufficientStake)?;

			Validators::<T>::insert(validator, remaining_stake);

			TotalValidatorStake::<T>::mutate(validator, |total| {
				*total = total.checked_sub(&slash_amount).unwrap_or_else(Zero::zero);
			});

			TotalStaked::<T>::mutate(|total| {
				*total = total.checked_sub(&slash_amount).unwrap_or_else(Zero::zero);
			});

			let _ = T::Currency::slash_reserved(validator, slash_amount);
			
			Ok(())
		}
	}
}