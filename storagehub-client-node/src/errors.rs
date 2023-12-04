use std::{str::Utf8Error, string::String};
use subxt::error::{DispatchError, MetadataError};
use thiserror::Error;

/// Crunch specific error messages
#[derive(Error, Debug)]
pub enum StorageHubError {
    #[error("Subxt error: {0}")]
    SubxtError(#[from] subxt::Error),
    #[error("Codec error: {0}")]
    CodecError(#[from] codec::Error),
    #[error("Utf8 error: {0}")]
    Utf8Error(#[from] Utf8Error),
    #[error("Metadata error: {0}")]
    MetadataError(#[from] MetadataError),
    #[error("Dispatch error: {0}")]
    DispatchError(#[from] DispatchError),
    #[error("Subscription finished")]
    SubscriptionFinished,
    #[error("ParseError error: {0}")]
    ParseError(#[from] url::ParseError),
    #[error("Other error: {0}")]
    Other(String),
}

/// Convert &str to CrunchError
impl From<&str> for StorageHubError {
    fn from(error: &str) -> Self {
        StorageHubError::Other(error.into())
    }
}
