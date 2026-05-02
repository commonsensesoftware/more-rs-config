mod error;

pub use error::Error;

#[cfg(feature = "binder")]
mod r#impl;

#[cfg(feature = "binder")]
pub use r#impl::{bind, from};
