use crate::{Configuration, ConfigurationProvider};
use std::{fmt::Debug, ops::Deref};

/// Represents the root of a [configuration](trait.Configuration.html) hierarchy.
pub trait ConfigurationRoot: Configuration + Deref<Target = dyn Configuration> + Debug {
    /// Force the configuration values to be reloaded from the underlying [provider](trait.ConfigurationProvider.html).
    fn reload(&mut self);

    /// Gets the [providers](trait.ConfigurationProvider.html) for this configuration.
    fn providers(&self) -> &[Box<dyn ConfigurationProvider>];

    /// Converts the [ConfigurationRoot](trait.ConfigurationRoot.html) into a [Configuration](trait.Configuration.html).
    fn as_config(&self) -> Box<dyn Configuration>;
}