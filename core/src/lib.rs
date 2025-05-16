#![deny(clippy::unwrap_used, clippy::expect_used)]
// Exposing generate_passsword
mod algorithms;
pub mod crack;
pub mod dump_hashes;
pub mod dump_rainbow_table;
pub mod generate_hashes;
pub mod generate_passwords;
pub mod generate_rainbow_table;
pub mod hash;
mod radix_type;
pub mod reduction;
pub mod table;
pub mod utils;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum MyError {
    #[error("file open error")]
    FileOpen {
        #[from]
        source: std::io::Error,
    },
    #[error("thread join error")]
    Send {
        #[from]
        source: crossbeam_channel::SendError<String>,
    },

    #[error("thread join error")]
    ThreadJoin,
}

/// This enum groups I/O, thread, algorithm, and input-related errors into
/// descriptive variants, each with a helpful error message.
///
/// # Variants
///
/// - `FileOpen`, `FileRead`, `CreateFile`, `WriteError`: File I/O related errors.
/// - `ThreadJoin`, `ThreadError`: Threading issues.
/// - `SendError`: Channel communication failure.
/// - `InvalidThreadCount`, `InvalidInput`, `InvalidHashLength`: Parameter validation errors.
/// - `UnknownAlgorithm`: Unsupported or unrecognized hash algorithm.
/// - `CustomError`: General-purpose error for custom messages.
#[derive(Debug, Error)]
pub enum HashassinError {
    #[error("File open error: {0}")]
    FileOpen(String),

    #[error("File Read error: {0}")]
    FileRead(String),

    #[error("File create error: {0}")]
    CreateFile(String),

    #[error("Thread join failed: {0}")]
    ThreadJoin(String),

    #[error("The number of threads must be greater than zero.")]
    InvalidThreadCount,

    #[error("Failed to send data over the channel: {0}")]
    SendError(String),

    #[error("Failed to create thread: {0}")]
    ThreadError(String),

    #[error("Failed to Write error: {0}")]
    WriteError(String),

    #[error("{0}")]
    CustomError(String),

    #[error("Invalid Input: {0}")]
    InvalidInput(String),

    #[error("Input file as invalid Format: {0}")]
    InvalidFormat(String),

    #[error("Unknown Algorithm: {0}")]
    UnknownAlgorithm(String),

    #[error("Invalid hash length: {0}")]
    InvalidHashLength(String),
}
