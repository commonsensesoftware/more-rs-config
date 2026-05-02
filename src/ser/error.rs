use thiserror::Error;

/// Represents the serialization errors that can occur.
#[derive(Error, Debug)]
pub enum Error {
    /// A custom error message from serde.
    #[error("{0}")]
    Custom(String),

    /// A map key could not be serialized to a string.
    #[error("map keys must be serializable to strings")]
    NonStringKey,
}

#[cfg(feature = "typed")]
impl serde::ser::Error for Error {
    #[inline]
    fn custom<T: std::fmt::Display>(msg: T) -> Self {
        Self::Custom(msg.to_string())
    }
}
