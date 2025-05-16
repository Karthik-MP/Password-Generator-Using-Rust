#![deny(clippy::unwrap_used, clippy::expect_used)]
pub mod cache_memory;
pub mod compute_threads;
pub mod crack_hashes;
pub mod save_rainbow_table;
pub mod server;
use core::str;
use std::io;

use thiserror::Error;
pub enum MyError {}

#[derive(Debug, Error)]
pub enum ServerError {
    IoError(io::Error),
    Utf8Error(str::Utf8Error),
    InvalidMagicWord,
    MetadataError,
    CacheError,
    CachePoisonedError,
    InvalidAlgorithm,
    MutexError,
    UnableUnwrapArc,
    ChainError(io::Error),
    NoPasswordsFound,
    PasswordNotFoundInCache,
    NoRainbowTableFound,
    BindingError,
}

impl std::fmt::Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ServerError::IoError(err) => write!(f, "IO error: {}", err),
            ServerError::Utf8Error(err) => write!(f, "UTF-8 error: {}", err),
            ServerError::InvalidMagicWord => write!(f, "Invalid magic word"),
            ServerError::MetadataError => write!(f, "Metadata error"),
            ServerError::CacheError => write!(f, "Cache error"),
            ServerError::InvalidAlgorithm => write!(f, "Invalid algorithm"),
            ServerError::ChainError(error) => {
                write!(f, "Chain error: {}", error)
            }
            ServerError::MutexError => write!(f, "Error Reading Cache:: Mutex error"),
            ServerError::NoPasswordsFound => write!(f, "No passwords found"),
            ServerError::UnableUnwrapArc => write!(f, "Unable to unwrap Arc"),
            ServerError::NoRainbowTableFound => {
                write!(f, "Rainbow Table not found for the given hash file")
            }
            ServerError::CachePoisonedError => write!(f, "Cache poisoned error"),
            ServerError::PasswordNotFoundInCache => write!(f, "Password not found in cache"),
            ServerError::BindingError => write!(f, "Could not bind server to address"),
        }
    }
}

impl From<io::Error> for ServerError {
    fn from(err: io::Error) -> ServerError {
        ServerError::IoError(err)
    }
}

impl From<str::Utf8Error> for ServerError {
    fn from(err: str::Utf8Error) -> ServerError {
        ServerError::Utf8Error(err)
    }
}
