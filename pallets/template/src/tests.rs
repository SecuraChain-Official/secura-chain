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