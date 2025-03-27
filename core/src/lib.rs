// Exposing generate_passsword
pub mod dump_hashes;
pub mod generate_hashes;
pub mod generate_passwords;

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

#[derive(Debug, Error)]
pub enum HashassinError {
    #[error("File open error: {0}")]
    FileOpen(String),

    #[error("File open error: {0}")]
    CreateFile(String),

    #[error("Thread join failed: {0}")]
    ThreadJoin(String),

    #[error("The number of threads must be greater than zero.")]
    InvalidThreadCount,

    #[error("Failed to send to printer thread: {0}")]
    SendError(String),

    #[error("Failed to create thread: {0}")]
    ThreadError(String),

    #[error("Failed to Write error: {0}")]
    WriteError(String),

    #[error("{0}")]
    CustomError(String),
}
