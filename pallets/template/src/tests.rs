use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

#[test]
fn register_validator_works() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		
		// Register validator with 500 tokens
		assert_ok!(TemplateModule::register_validator(RuntimeOrigin::signed(1), 500));
		
		// Check validator is registered
		assert_eq!(TemplateModule::validators(1), 500);
		
		// Check validator count
		assert_eq!(TemplateModule::validator_count(), 1);
		
		// Check total staked
		assert_eq!(TemplateModule::total_staked(), 500);
		
		// Check total validator stake
		assert_eq!(TemplateModule::total_validator_stake(1), 500);
		
		// System emits event
		System::assert_has_event(Event::ValidatorRegistered(1, 500).into());
	});
}

#[test]
fn register_validator_fails_when_already_registered() {
	new_test_ext().execute_with(|| {
		// Register as validator
		assert_ok!(TemplateModule::register_validator(RuntimeOrigin::signed(1), 500));
		
		// Try to register again
		assert_noop!(
			TemplateModule::register_validator(RuntimeOrigin::signed(1), 600),
			Error::<Test>::AlreadyValidator
		);
	});
}

#[test]
fn register_validator_fails_with_insufficient_stake() {
	new_test_ext().execute_with(|| {
		// Try to register with less than minimum stake
		assert_noop!(
			TemplateModule::register_validator(RuntimeOrigin::signed(1), 50),
			Error::<Test>::StakeBelowMinimum
		);
	});
}

#[test]
fn register_validator_fails_with_insufficient_balance() {
	new_test_ext().execute_with(|| {
		// Try to register with more than balance
		assert_noop!(
			TemplateModule::register_validator(RuntimeOrigin::signed(1), 2000),
			Error::<Test>::InsufficientBalance
		);
	});
}

#[test]
fn remove_validator_works() {
	new_test_ext().execute_with(|| {
		// Register as validator
		assert_ok!(TemplateModule::register_validator(RuntimeOrigin::signed(1), 500));
		
		// Remove validator status
		assert_ok!(TemplateModule::remove_validator(RuntimeOrigin::signed(1)));
		
		// Check validator is removed
		assert_eq!(TemplateModule::validators(1), 0);
		
		// Check validator count
		assert_eq!(TemplateModule::validator_count(), 0);
		
		// Check total staked
		assert_eq!(TemplateModule::total_staked(), 0);
	});
}

#[test]
fn remove_validator_fails_when_not_validator() {
	new_test_ext().execute_with(|| {
		// Try to remove validator status when not a validator
		assert_noop!(
			TemplateModule::remove_validator(RuntimeOrigin::signed(1)),
			Error::<Test>::NotValidator
		);
	});
}

#[test]
fn nominate_works() {
	new_test_ext().execute_with(|| {
		// Register validator
		assert_ok!(TemplateModule::register_validator(RuntimeOrigin::signed(1), 500));
		
		// Nominate validator
		assert_ok!(TemplateModule::nominate(RuntimeOrigin::signed(2), 1, 100));
		
		// Check nomination is stored
		let nominations = TemplateModule::nominators(2);
		assert_eq!(nominations.len(), 1);
		assert_eq!(nominations[0].validator, 1);
		assert_eq!(nominations[0].amount, 100);
		
		// Check total validator stake is updated
		assert_eq!(TemplateModule::total_validator_stake(1), 600);
		
		// Check total staked is updated
		assert_eq!(TemplateModule::total_staked(), 600);
		
		// System emits event
		System::assert_has_event(Event::Nomination(2, 1, 100).into());
	});
}

#[test]
fn nominate_fails_when_validator_not_exists() {
	new_test_ext().execute_with(|| {
		// Try to nominate non-existent validator
		assert_noop!(
			TemplateModule::nominate(RuntimeOrigin::signed(2), 1, 100),
			Error::<Test>::NotValidator
		);
	});
}

#[test]
fn nominate_fails_with_insufficient_amount() {
	new_test_ext().execute_with(|| {
		// Register validator
		assert_ok!(TemplateModule::register_validator(RuntimeOrigin::signed(1), 500));
		
		// Try to nominate with less than minimum
		assert_noop!(
			TemplateModule::nominate(RuntimeOrigin::signed(2), 1, 5),
			Error::<Test>::NominationBelowMinimum
		);
	});
}

#[test]
fn nominate_fails_when_already_nominated() {
	new_test_ext().execute_with(|| {
		// Register validator
		assert_ok!(TemplateModule::register_validator(RuntimeOrigin::signed(1), 500));
		
		// Nominate validator
		assert_ok!(TemplateModule::nominate(RuntimeOrigin::signed(2), 1, 100));
		
		// Try to nominate again
		assert_noop!(
			TemplateModule::nominate(RuntimeOrigin::signed(2), 1, 100),
			Error::<Test>::AlreadyNominated
		);
	});
}

#[test]
fn withdraw_nomination_works() {
	new_test_ext().execute_with(|| {
		// Register validator
		assert_ok!(TemplateModule::register_validator(RuntimeOrigin::signed(1), 500));
		
		// Nominate validator
		assert_ok!(TemplateModule::nominate(RuntimeOrigin::signed(2), 1, 100));
		
		// Withdraw nomination
		assert_ok!(TemplateModule::withdraw_nomination(RuntimeOrigin::signed(2), 1));
		
		// Check nomination is removed
		let nominations = TemplateModule::nominators(2);
		assert_eq!(nominations.len(), 0);
		
		// Check total validator stake is updated
		assert_eq!(TemplateModule::total_validator_stake(1), 500);
		
		// Check total staked is updated
		assert_eq!(TemplateModule::total_staked(), 500);
		
		// System emits event
		System::assert_has_event(Event::NominationWithdrawn(2, 1, 100).into());
	});
}

#[test]
fn withdraw_nomination_fails_when_not_nominated() {
	new_test_ext().execute_with(|| {
		// Register validator
		assert_ok!(TemplateModule::register_validator(RuntimeOrigin::signed(1), 500));
		
		// Try to withdraw non-existent nomination
		assert_noop!(
			TemplateModule::withdraw_nomination(RuntimeOrigin::signed(2), 1),
			Error::<Test>::NominationNotFound
		);
	});
}

#[test]
fn rewards_distribution_works() {
	new_test_ext().execute_with(|| {
		// Register validator
		assert_ok!(TemplateModule::register_validator(RuntimeOrigin::signed(1), 500));
		
		// Nominate validator
		assert_ok!(TemplateModule::nominate(RuntimeOrigin::signed(2), 1, 500));
		
		// Advance blocks to trigger reward distribution
		System::set_block_number(2);
		TemplateModule::on_initialize(2);
		
		// Check rewards were distributed
		assert!(TemplateModule::pending_rewards(1) > 0);
		assert!(TemplateModule::pending_rewards(2) > 0);
		
		// Claim rewards for validator
		let validator_rewards = TemplateModule::pending_rewards(1);
		assert_ok!(TemplateModule::claim_rewards(RuntimeOrigin::signed(1)));
		
		// Check rewards were claimed
		assert_eq!(TemplateModule::pending_rewards(1), 0);
		
		// Claim rewards for nominator
		let nominator_rewards = TemplateModule::pending_rewards(2);
		assert_ok!(TemplateModule::claim_rewards(RuntimeOrigin::signed(2)));
		
		// Check rewards were claimed
		assert_eq!(TemplateModule::pending_rewards(2), 0);
		
		// System emits events
		System::assert_has_event(Event::RewardsClaimed(1, validator_rewards).into());
		System::assert_has_event(Event::RewardsClaimed(2, nominator_rewards).into());
	});
}

#[test]
fn claim_rewards_fails_when_no_rewards() {
	new_test_ext().execute_with(|| {
		// Try to claim rewards when none exist
		assert_noop!(
			TemplateModule::claim_rewards(RuntimeOrigin::signed(1)),
			Error::<Test>::NoRewards
		);
	});
}