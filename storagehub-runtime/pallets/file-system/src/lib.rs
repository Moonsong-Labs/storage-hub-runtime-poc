#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://docs.substrate.io/reference/frame-pallets/>
pub use pallet::*;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;
pub mod weights;
pub use weights::*;

mod types;
mod utils;

#[frame_support::pallet]
pub mod pallet {
	use super::{types::*, *};
	use frame_support::{
		dispatch::{fmt::Debug, HasCompact},
		pallet_prelude::*,
		sp_runtime::traits::{AtLeast32Bit, CheckEqual, MaybeDisplay, SimpleBitOps},
	};
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	pub struct Pallet<T>(_);

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
		/// Type representing the weight of this pallet
		type WeightInfo: WeightInfo;

		/// Type to access the Identity Pallet, where BSPs are registered.
		type BspsRegistry: pallet_identity::IdentityInterface<AccountId = Self::AccountId>;

		/// The type for Content IDs of files, generally a hash.
		type Fingerprint: Parameter
			+ Member
			+ MaybeSerializeDeserialize
			+ Debug
			+ MaybeDisplay
			+ SimpleBitOps
			+ Ord
			+ Default
			+ Copy
			+ CheckEqual
			+ AsRef<[u8]>
			+ AsMut<[u8]>
			+ MaxEncodedLen;

		/// The unit for representing the size of a file.
		type StorageCount: Parameter
			+ Member
			+ MaybeSerializeDeserialize
			+ Debug
			+ Default
			+ MaybeDisplay
			+ AtLeast32Bit
			+ Copy
			+ MaxEncodedLen
			+ HasCompact;

		/// The threshold that the randomness criteria operation result should
		/// meet, for the caller to instantly be eligible as BSP for that file.
		type AssignmentThreshold: Parameter
			+ Member
			+ MaybeSerializeDeserialize
			+ Debug
			+ Default
			+ MaybeDisplay
			+ AtLeast32Bit
			+ Copy
			+ MaxEncodedLen
			+ HasCompact;

		/// The maximum number of BSPs per file.
		#[pallet::constant]
		type MaxBsps: Get<u32>;

		/// The maximum size of a file path in bytes.
		#[pallet::constant]
		type MaxFilePathSize: Get<u32>;

		/// The maximum size of a libp2p multiaddress in bytes.
		#[pallet::constant]
		type MaxMultiAddressSize: Get<u32>;

		/// The minimum threshold that the randomness criteria operation result
		/// should meet, for the caller to instantly be eligible as BSP for that
		/// file. This minimum threshold should decrease when more BSPs are
		/// added to the system, and increased if BSPs leave the system.
		#[pallet::constant]
		type MinBspsAssignmentThreshold: Get<Self::AssignmentThreshold>;
	}

	#[pallet::storage]
	pub type StorageRequests<T: Config> =
		StorageMap<_, Blake2_128Concat, FileLocation<T>, FileMetadata<T>>;

	#[pallet::storage]
	pub type FilesMapping<T: Config> =
		StorageMap<_, Blake2_128Concat, FileLocation<T>, FileMetadata<T>>;

	#[pallet::storage]
	#[pallet::getter(fn total_used_bsps_storage)]
	pub type TotalUsedBspStorage<T: Config> = StorageValue<_, <T as Config>::StorageCount>;

	#[pallet::storage]
	#[pallet::getter(fn current_assignment_threshold)]
	pub type CurrentAssignmentThreshold<T: Config> =
		StorageValue<_, <T as Config>::AssignmentThreshold>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		NewStorageRequest {
			who: T::AccountId,
			location: FileLocation<T>,
			fingerprint: Fingerprint<T>,
			size: StorageCount<T>,
			sender_multiaddress: MultiAddress<T>,
		},

		NewBspVolunteer {
			who: T::AccountId,
			location: FileLocation<T>,
			fingerprint: Fingerprint<T>,
			bsp_multiaddress: MultiAddress<T>,
		},
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Trying to register a storage request for a file that is already registered.
		StorageRequestAlreadyRegistered,

		/// Trying to volunteer as BSP for a storage request, when sender is not a registered BSP.
		NotBsp,

		/// Trying to operate over a non-existing storage request.
		StorageRequestNotRegistered,

		/// Trying to volunteer a BSP for a storage request, when that BSP is already registered
		/// for that storage request.
		BspAlreadyRegistered,
	}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialise as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T: Config> Pallet<T> {
		// TODO: Document
		#[pallet::call_index(0)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn request_storage(
			origin: OriginFor<T>,
			location: FileLocation<T>,
			fingerprint: Fingerprint<T>,
			size: StorageCount<T>,
			sender_multiaddress: MultiAddress<T>,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;

			// Perform validations and register storage request.
			Self::do_request_storage(location.clone(), fingerprint)?;

			// Emit new storage request event.
			Self::deposit_event(Event::NewStorageRequest {
				who,
				location,
				fingerprint,
				size,
				sender_multiaddress,
			});

			Ok(())
		}

		// TODO: Document
		#[pallet::call_index(1)]
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1).ref_time())]
		pub fn bsp_volunteer(
			origin: OriginFor<T>,
			location: FileLocation<T>,
			fingerprint: Fingerprint<T>,
			bsp_multiaddress: MultiAddress<T>,
		) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			let who = ensure_signed(origin)?;

			// Perform validations and register Storage Provider as BSP for file.
			Self::do_bsp_volunteer(who.clone(), location.clone())?;

			// Emit new BSP volunteer event.
			Self::deposit_event(Event::NewBspVolunteer {
				who,
				location,
				fingerprint,
				bsp_multiaddress,
			});

			Ok(())
		}
	}
}
