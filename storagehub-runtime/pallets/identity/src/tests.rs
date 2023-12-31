use crate::{mock::*, Error, Event};
use frame_support::{assert_noop, assert_ok};
use sp_runtime::DispatchError;

#[test]
fn register_user_success() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		// Register user.
		assert_ok!(Identity::register_user(RuntimeOrigin::root(), 1));

		// Check that event was emitted.
		System::assert_last_event(Event::NewUser { user: 1 }.into());

		// Check storage.
		assert_eq!(Identity::users(1), Some(()));
	});
}

#[test]
fn register_user_not_root() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		// Register user.
		assert_noop!(
			Identity::register_user(RuntimeOrigin::signed(1), 1),
			DispatchError::BadOrigin
		);
	});
}

#[test]
fn register_user_max_reached() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		// Register MAX_USERS users.
		for i in 0..MAX_USERS {
			assert_ok!(Identity::register_user(RuntimeOrigin::root(), i as u64));
		}

		// Try to register one more user.
		assert_noop!(
			Identity::register_user(RuntimeOrigin::root(), MAX_USERS.try_into().unwrap()),
			Error::<Test>::MaximumOfUsersReached
		);
	});
}

#[test]
fn remove_user_success() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		// Register user.
		assert_ok!(Identity::register_user(RuntimeOrigin::root(), 1));

		// Remove user.
		assert_ok!(Identity::remove_user(RuntimeOrigin::root(), 1));

		// Check that event was emitted.
		System::assert_last_event(Event::RemovedUser { user: 1 }.into());

		// Check storage.
		assert_eq!(Identity::users(1), None);
	});
}

#[test]
fn remove_user_not_root() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		// Remove user.
		assert_noop!(Identity::remove_user(RuntimeOrigin::signed(1), 1), DispatchError::BadOrigin);
	});
}

#[test]
fn remove_user_not_registered() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		// Remove user.
		assert_noop!(Identity::remove_user(RuntimeOrigin::root(), 1), Error::<Test>::NotRegistered);
	});
}
