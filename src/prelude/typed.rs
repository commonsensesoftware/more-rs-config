use crate::{typed, Builder};
use serde::Serialize;

/// Defines typed configuration extension methods for a [configuration builder](Builder).
pub trait TypedExt: Sized {
    /// Adds a typed configuration source using a serializable type.
    ///
    /// # Arguments
    ///
    /// * `value` - The `Serialize`-able value to use as a configuration source
    fn add_typed<T: Serialize + Send + Sync + 'static>(self, value: T) -> Self;
}

impl TypedExt for Builder {
    fn add_typed<T: Serialize + Send + Sync + 'static>(mut self, value: T) -> Self {
        self.add(typed::Provider::new(value));
        self
    }
}
