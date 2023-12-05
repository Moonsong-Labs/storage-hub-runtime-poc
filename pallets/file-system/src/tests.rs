use crate::{mock::*, types::FileLocation, Event};
use frame_support::assert_ok;
use sp_runtime::{
	traits::{BlakeTwo256, Hash},
	BoundedVec,
};

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
			BoundedVec::try_from(vec![1]).unwrap(),
		));

		// Assert that the correct event was deposited
		System::assert_last_event(
			Event::NewStorageRequest {
				who: 1,
				location,
				content_id,
				size: 4,
				sender_multiaddress: BoundedVec::try_from(vec![1]).unwrap(),
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
			BoundedVec::try_from(vec![1]).unwrap(),
		));

		// Dispatch BSP volunteer.
		assert_ok!(FileSystem::bsp_volunteer(
			bsp.clone(),
			location.clone(),
			content_id.clone(),
			BoundedVec::try_from(vec![2]).unwrap()
		));

		// Assert that the correct event was deposited
		System::assert_last_event(
			Event::NewBspVolunteer {
				who: 2,
				location,
				content_id,
				bsp_multiaddress: BoundedVec::try_from(vec![2]).unwrap(),
			}
			.into(),
		);
	});
}
