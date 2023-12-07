use codec::{Decode, Encode};
use frame_support::{ensure, pallet_prelude::DispatchResult, sp_runtime::BoundedVec, traits::Get};
use pallet_identity::IdentityInterface;
use scale_info::prelude::vec::Vec;
use sp_runtime::{
	traits::{BlakeTwo256, Hash},
	SaturatedConversion, Saturating,
};

use crate::{
	pallet,
	types::{FileLocation, FileMetadata, Fingerprint},
	Config, Error, FilesMapping, Pallet, StorageRequests,
};

macro_rules! expect_or_err {
	($optional:expr, $error_msg:expr, $error_type:path) => {
		match $optional {
			Some(value) => value,
			None => {
				#[cfg(test)]
				unreachable!($error_msg);

				#[allow(unreachable_code)]
				{
					Err($error_type)?
				}
			},
		}
	};
}

impl<T> Pallet<T>
where
	T: pallet::Config,
{
	pub fn do_request_storage(
		location: FileLocation<T>,
		content_id: Fingerprint<T>,
	) -> DispatchResult {
		// TODO: Perform various checks of users funds, storage capacity, etc.
		// TODO: Not relevant for PoC.

		// Construct file metadata.
		let file_metadata = FileMetadata::<T> {
			requested_at: <frame_system::Pallet<T>>::block_number(),
			fingerprint: content_id.clone(),
			bsps: BoundedVec::default(),
			is_public: true,
		};

		// Check that storage request is not already registered.
		ensure!(
			!<StorageRequests<T>>::contains_key(&location),
			Error::<T>::StorageRequestAlreadyRegistered
		);

		// Register storage request.
		<StorageRequests<T>>::insert(&location, file_metadata);

		Ok(())
	}

	pub fn do_bsp_volunteer(
		who: T::AccountId,
		location: FileLocation<T>,
		fingerprint: Fingerprint<T>,
	) -> DispatchResult {
		// TODO: Perform various checks of BSP staking, total capacity, etc.
		// TODO: Not relevant for PoC.

		// Check that sender is a registered storage provider.
		ensure!(<T as Config>::BspsRegistry::get_user(who.clone()).is_some(), Error::<T>::NotBsp);

		// Check that the storage request exists.
		ensure!(
			<StorageRequests<T>>::contains_key(&location),
			Error::<T>::StorageRequestNotRegistered
		);

		// Get storage request metadata.
		let mut file_metadata = expect_or_err!(
			<StorageRequests<T>>::get(&location),
			"Storage request should exist",
			Error::<T>::StorageRequestNotRegistered
		);

		// Check that BSP is not already registered for this storage request.
		ensure!(!file_metadata.bsps.contains(&who), Error::<T>::BspAlreadyRegistered);

		// Check that the threshold value is high enough to qualify as BSP for the storage request.
		let who_bytes = BlakeTwo256::hash(&who.encode()).0;
		let threshold = calculate_xor(fingerprint.as_ref().try_into().unwrap(), &who_bytes);

		let threshold = T::AssignmentThreshold::decode(&mut &threshold[..])
			.map_err(|_| Error::<T>::FailedToDecodeThreshold)?;

		let blocks_since_requested = <frame_system::Pallet<T>>::block_number()
			.saturating_sub(file_metadata.requested_at)
			.saturated_into::<u32>();

		// Rate multiplier is 10,000.
		// This can probably be exposed as a configurable parameter.
		let rate_increase = blocks_since_requested
			.saturating_mul(10_000u32)
			.saturated_into::<T::AssignmentThreshold>();

		let min_threshold = rate_increase.saturating_add(T::MinBspsAssignmentThreshold::get());

		ensure!(threshold <= min_threshold, Error::<T>::ThresholdTooLow);

		// Add BSP to storage request metadata.
		file_metadata
			.bsps
			.try_push(who.clone())
			.map_err(|_| Error::<T>::BspAlreadyRegistered)?;
		<StorageRequests<T>>::set(&location, Some(file_metadata.clone()));

		// Check if storage request is in FilesMapping now that it has at least one BSP.
		if !<FilesMapping<T>>::contains_key(&location) {
			// Add storage request to FilesMapping.
			<FilesMapping<T>>::insert(&location, file_metadata.clone());
		}

		// Check if maximum number of BSPs has been reached.
		if file_metadata.bsps.len() == T::MaxBsps::get() as usize {
			// Clear storage request from StorageRequests.
			<StorageRequests<T>>::remove(&location);
		}

		Ok(())
	}
}

fn calculate_xor(fingerprint: &[u8; 32], bsp: &[u8; 32]) -> Vec<u8> {
	let mut xor_result = Vec::with_capacity(32);
	for i in 0..32 {
		xor_result.push(fingerprint[i] ^ bsp[i]);
	}

	xor_result
}
