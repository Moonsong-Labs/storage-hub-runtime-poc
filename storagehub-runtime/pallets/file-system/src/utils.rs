use frame_support::{ensure, pallet_prelude::DispatchResult, sp_runtime::BoundedVec, traits::Get};
use pallet_identity::IdentityInterface;

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
			content_id: content_id.clone(),
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

	pub fn do_bsp_volunteer(who: T::AccountId, location: FileLocation<T>) -> DispatchResult {
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

		// TODO: Check if BSP qualifies for volunteering based on randomness criteria.

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
