use crate::{de, ser};
use std::{fmt::Debug, path::PathBuf};
use thiserror::Error;

/// Defines the possible configuration errors.
#[derive(Error, Debug)]
pub enum Error {
    /// Indicates a custom configuration error has occurred.
    #[error("{0}")]
    Custom(String),

    /// Indicates an invalid configuration file has been provided.
    #[error("{message}")]
    InvalidFile {
        /// Gets the error message.
        message: String,

        /// Gets the path of the file being loaded.
        path: PathBuf,
    },

    /// Indicates a required configuration file is missing.
    #[error("The configuration file '{0}' was not found, but is required.")]
    MissingFile(PathBuf),

    /// Indicates that a reification operation failed.
    #[error(transparent)]
    ReifyFailed(#[from] de::Error),

    /// Indicates that a serialization operation failed.
    #[error(transparent)]
    SerializeFailed(#[from] ser::Error),

    /// Indicates that an unknown [error](std::error::Error) occurred.
    #[error(transparent)]
    Unknown(#[from] Box<dyn std::error::Error>),
}

impl Error {
    /// Creates a new [custom](Self::Custom) error.
    ///
    /// # Arguments
    ///
    /// * `message` - The error message
    #[inline]
    pub fn custom(message: impl Into<String>) -> Self {
        Self::Custom(message.into())
    }

    /// Creates a new [unknown](Self::Unknown) error.
    ///
    /// # Arguments
    ///
    /// * `error` - The unknown [error](std::error::Error)
    #[inline]
    pub fn unknown(error: impl std::error::Error + 'static) -> Self {
        Self::Unknown(Box::new(error))
    }
}
