use crate::{ser, Settings};
use serde::Serialize;

/// Represents a [configuration provider](crate::Provider) that serializes a [serializable](Serialize) as a set of
/// configuration key/ pairs.
pub struct Provider<T: Serialize>(T);

impl<T: Serialize> Provider<T> {
    /// Initializes a new typed configuration provider.
    ///
    /// # Arguments
    ///
    /// * `value` - The `Serialize`-able value to use as a configuration source
    #[inline]
    pub fn new(value: T) -> Self {
        Self(value)
    }
}

impl<T: Serialize + Send + Sync> crate::Provider for Provider<T> {
    #[inline]
    fn name(&self) -> &str {
        "Typed"
    }

    fn load(&self, settings: &mut Settings) -> crate::Result {
        ser::into(&self.0, settings)?;
        Ok(())
    }
}
