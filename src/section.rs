use crate::{Configuration, Value};
use std::{borrow::Borrow, ops::Deref};

/// Defines the behavior for a section of application configuration values.
pub trait ConfigurationSection:
    Configuration
    + AsRef<dyn Configuration>
    + Borrow<dyn Configuration>
    + Deref<Target = dyn Configuration>
{
    /// Gets the key this section occupies in its parent.
    fn key(&self) -> &str;

    /// Gets the full path to this section within the [`Configuration`](crate::Configuration).
    fn path(&self) -> &str;

    /// Gets the section value.
    fn value(&self) -> Value;

    /// Converts the [`ConfigurationSection`] into a [`Configuration`](crate::Configuration).
    fn as_config(&self) -> Box<dyn Configuration>;
}

pub mod ext {

    use super::*;

    /// Defines extension methods for [`ConfigurationSection`].
    pub trait ConfigurationSectionExtensions {
        /// Gets a value indicating whether the configuration section exists.
        ///
        /// # Remarks
        ///
        /// A configuration section is considered nonexistent if it has no
        /// value and no children
        fn exists(&self) -> bool;
    }

    impl ConfigurationSectionExtensions for dyn ConfigurationSection + '_ {
        fn exists(&self) -> bool {
            !self.value().is_empty() || !self.children().is_empty()
        }
    }

    impl<T: ConfigurationSection> ConfigurationSectionExtensions for T {
        fn exists(&self) -> bool {
            !self.value().is_empty() || !self.children().is_empty()
        }
    }
}
