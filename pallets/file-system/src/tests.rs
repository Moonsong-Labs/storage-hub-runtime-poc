use crate::{mock::*, types::FileLocation, Event};
use frame_support::assert_ok;
use sp_runtime::traits::{BlakeTwo256, Hash};

#[test]
fn request_storage_success() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let user = RuntimeOrigin::signed(1);
		let location = FileLocation::<Test>::try_from(b"test".to_vec()).unwrap();
		let file_content = b"test".to_vec();
		let content_id = BlakeTwo256::hash(&file_content);

		// Dispatch storage request.
		assert_ok!(FileSystem::request_storage(
			user.clone(),
			location.clone(),
			content_id.clone(),
			4,
			1
		));

		// Assert that the correct event was deposited
		System::assert_last_event(
			Event::NewStorageRequest {
				who: 1,
				location,
				content_id,
				size: 4,
				sender_multiaddress: 1,
			}
			.into(),
		);
	});
}

#[test]
fn bsp_volunteer_success() {
	new_test_ext().execute_with(|| {
		// Go past genesis block so events get deposited
		System::set_block_number(1);

		let user = RuntimeOrigin::signed(1);
		let bsp = RuntimeOrigin::signed(2);
		let location = FileLocation::<Test>::try_from(b"test".to_vec()).unwrap();
		let file_content = b"test".to_vec();
		let content_id = BlakeTwo256::hash(&file_content);

		// Register BSP in Identity Pallet.
		assert_ok!(Identity::register_user(RuntimeOrigin::root(), 2));

		// Dispatch storage request.
		assert_ok!(FileSystem::request_storage(
			user.clone(),
			location.clone(),
			content_id.clone(),
			4,
			1
		));

		// Dispatch BSP volunteer.
		assert_ok!(FileSystem::bsp_volunteer(bsp.clone(), location.clone(), content_id.clone(), 2));

		// Assert that the correct event was deposited
		System::assert_last_event(
			Event::NewBspVolunteer { who: 2, location, content_id, bsp_multiaddress: 2 }.into(),
		);
	});
}
