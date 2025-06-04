use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};

#[test]
fn register_validator_works() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);
		
		// Register as validator
		assert_ok!(TemplateModule::register_validator(RuntimeOrigin::signed(1)));
		
		// Check validator is registered
		assert_eq!(TemplateModule::validators(1), true);
		
		// Check validator count
		assert_eq!(TemplateModule::validator_count(), 1);
		
		// System emits event
		System::assert_has_event(Event::ValidatorRegistered(1).into());
	});
}

#[test]
fn register_validator_fails_when_already_registered() {
	new_test_ext().execute_with(|| {
		// Register as validator
		assert_ok!(TemplateModule::register_validator(RuntimeOrigin::signed(1)));
		
		// Try to register again
		assert_noop!(
			TemplateModule::register_validator(RuntimeOrigin::signed(1)),
			Error::<Test>::AlreadyValidator
		);
	});
}

#[test]
fn remove_validator_works() {
	new_test_ext().execute_with(|| {
		// Register as validator
		assert_ok!(TemplateModule::register_validator(RuntimeOrigin::signed(1)));
		
		// Remove validator status
		assert_ok!(TemplateModule::remove_validator(RuntimeOrigin::signed(1)));
		
		// Check validator is removed
		assert_eq!(TemplateModule::validators(1), false);
		
		// Check validator count
		assert_eq!(TemplateModule::validator_count(), 0);
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