use thiserror::Error;

/// Represents the deserialization errors that can occur.
#[derive(Error, Debug, PartialEq)]
pub enum Error {
    /// Indicates a value is missing for a field.
    #[error("Missing value for field '{0}'")]
    MissingValue(&'static str),

    /// Indicates a custom error message
    #[error("{0}")]
    Custom(String),
}
