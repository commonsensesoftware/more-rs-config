use crate::{Configuration, Settings};

/// Defines the behavior of a merge operation between two objects.
pub trait Merge<T = Self> {
    /// Merges an object into current object.
    ///
    /// # Arguments
    ///
    /// * `other` - The other object to merge into the current object
    fn merge(&mut self, other: &T);
}

impl Merge<Configuration> for Settings {
    #[inline]
    fn merge(&mut self, other: &Configuration) {
        self.merge(&other.settings)
    }
}

impl Merge for Configuration {
    #[inline]
    fn merge(&mut self, other: &Self) {
        self.settings.merge(&other.settings)
    }
}

impl Merge<Settings> for Configuration {
    #[inline]
    fn merge(&mut self, other: &Settings) {
        self.settings.merge(other)
    }
}
