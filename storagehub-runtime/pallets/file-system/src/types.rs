use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::BoundedVec;
use frame_system::pallet_prelude::BlockNumberFor;
use scale_info::TypeInfo;

///
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug, PartialEq, Eq, Clone)]
#[scale_info(skip_type_params(T))]
pub struct FileMetadata<T: crate::Config> {
	pub requested_at: BlockNumberFor<T>,
	pub fingerprint: Fingerprint<T>,
	pub bsps: BoundedVec<StorageProviderId<T>, MaxBsps<T>>,
	pub is_public: bool,
}

/// A byte array representing the file path.
pub type FileLocation<T> = BoundedVec<u8, MaxFilePathSize<T>>;

/// A byte array representing the libp2p multiaddress.
pub type MultiAddress<T> = BoundedVec<u8, MaxMultiAddressSize<T>>;

/// Syntactic sugar for the MaxBsps type used in the FileSystem pallet.
pub type MaxBsps<T> = <T as crate::Config>::MaxBsps;

/// Syntactic sugar for the MaxFilePathSize type used in the FileSystem pallet.
pub type MaxFilePathSize<T> = <T as crate::Config>::MaxFilePathSize;

/// Syntactic sugar for the MaxMultiAddressSize type used in the FileSystem pallet.
pub type MaxMultiAddressSize<T> = <T as crate::Config>::MaxMultiAddressSize;

/// Syntactic sugar for the type ContentId used in the System pallet.
pub type Fingerprint<T> = <T as crate::Config>::Fingerprint;

/// Syntactic sugar for the type StorageProviderId used in the System pallet.
pub type StorageProviderId<T> = <T as frame_system::Config>::AccountId;

/// Syntactic sugar for the type StorageCount used in the System pallet.
pub type StorageCount<T> = <T as crate::Config>::StorageCount;
