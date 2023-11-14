use crate::{Configuration, ConfigurationProvider};
use std::{borrow::Borrow, fmt::Debug, ops::Deref};

/// Represents the root of a [configuration](trait.Configuration.html) hierarchy.
pub trait ConfigurationRoot:
    Configuration
    + AsRef<dyn Configuration>
    + Borrow<dyn Configuration>
    + Deref<Target = dyn Configuration>
    + Debug
{
    /// Force the configuration values to be reloaded from the underlying [provider](trait.ConfigurationProvider.html).
    ///
    /// # Returns
    ///
    /// True if the configuration values were reloaded from their underlying providers; otherwise, false.
    /// The configuration root must have exclusive access in order to reload from the underlying providers.
    /// Any owned or borrowed references to the root or any of its sections will prevent the reload from occurring.
    fn reload(&mut self) -> bool;

    /// Gets the [providers](trait.ConfigurationProvider.html) for this configuration.
    fn providers(&self) -> Box<dyn ConfigurationProviderIterator + '_>;

    /// Converts the [ConfigurationRoot](trait.ConfigurationRoot.html) into a [Configuration](trait.Configuration.html).
    fn as_config(&self) -> Box<dyn Configuration>;
}

/// Defines the behavior of an iterator over
/// [configuration providers](trait.ConfigurationProvider.html).
pub trait ConfigurationProviderIterator<'a>:
    Iterator<Item = &'a dyn ConfigurationProvider>
    + ExactSizeIterator<Item = &'a dyn ConfigurationProvider>
    + DoubleEndedIterator<Item = &'a dyn ConfigurationProvider>
{
}
