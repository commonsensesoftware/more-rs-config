mod error;

pub use error::Error;

#[cfg(feature = "typed")]
mod r#impl;

#[cfg(feature = "typed")]
pub use r#impl::into;
