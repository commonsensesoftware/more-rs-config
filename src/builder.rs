use crate::{ConfigurationRoot, ConfigurationSource};
use std::any::Any;
use std::collections::HashMap;

/// Defines the behavior used to build an application configuration.
pub trait ConfigurationBuilder {
    /// Gets a read-only key/value collection that can be used to share data between the
    /// [builder](trait.ConfigurationBuilder.html) and the registered [sources](trait.ConfigurationSource.html).
    fn properties(&self) -> &HashMap<String, Box<dyn Any>>;

    /// Gets [sources](trait.ConfigurationSource.html) used to obtain configuration values.
    fn sources(&self) -> &[Box<dyn ConfigurationSource>];

    /// Adds a new configuration source.
    ///
    /// # Arguments
    ///
    /// * `source` - The [configuration source](trait.ConfigurationSource.html) to add
    fn add(&mut self, source: Box<dyn ConfigurationSource>);

    /// Builds [configuration](trait.Configuration.html) with the keys and values from the set of registered
    /// [sources](trait.ConfigurationSource.html).
    fn build(&self) -> Box<dyn ConfigurationRoot>;
}
