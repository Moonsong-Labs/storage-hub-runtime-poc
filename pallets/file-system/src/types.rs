use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::BoundedVec;
use scale_info::TypeInfo;

///
#[derive(Encode, Decode, MaxEncodedLen, TypeInfo, Debug, PartialEq, Eq, Clone)]
#[scale_info(skip_type_params(T))]
pub struct FileMetadata<T: crate::Config> {
	pub content_id: ContentId<T>,
	pub bsps: BoundedVec<StorageProviderId<T>, MaxBsps<T>>,
	pub is_public: bool,
}

/// A byte array representing the file path.
pub type FileLocation<T> = BoundedVec<u8, MaxFilePathSize<T>>;

/// Syntactic sugar for the MaxBsps type used in the FileSystem pallet.
pub type MaxBsps<T> = <T as crate::Config>::MaxBsps;

/// Syntactic sugar for the MaxFilePathSize type used in the FileSystem pallet.
pub type MaxFilePathSize<T> = <T as crate::Config>::MaxFilePathSize;

/// Syntactic sugar for the type ContentId used in the System pallet.
pub type ContentId<T> = <T as crate::Config>::ContentId;

/// Syntactic sugar for the type StorageProviderId used in the System pallet.
pub type StorageProviderId<T> = <T as frame_system::Config>::AccountId;

/// Syntactic sugar for the type StorageCount used in the System pallet.
pub type StorageCount<T> = <T as crate::Config>::StorageCount;
