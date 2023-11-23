use crate::{ConfigurationRoot, ConfigurationSource, ReloadError};
use std::any::Any;
use std::collections::HashMap;

/// Defines the behavior used to build an application [`Configuration`](crate::Configuration).
pub trait ConfigurationBuilder {
    /// Gets a read-only key/value collection that can be used to share data between the
    /// [`ConfigurationBuilder`] and each registered [`ConfigurationSource`](crate::ConfigurationSource).
    fn properties(&self) -> &HashMap<String, Box<dyn Any>>;

    /// Gets the registered [`ConfigurationSource`](crate::ConfigurationSource) set used to obtain
    /// configuration values.
    fn sources(&self) -> &[Box<dyn ConfigurationSource>];

    /// Adds a new configuration source.
    ///
    /// # Arguments
    ///
    /// * `source` - The [`ConfigurationSource`](crate::ConfigurationSource) to add
    fn add(&mut self, source: Box<dyn ConfigurationSource>);

    /// Builds [`ConfigurationRoot`](crate::ConfigurationRoot) with the keys and values from the
    /// registered [`ConfigurationSource`](crate::ConfigurationSource) set.
    fn build(&self) -> Result<Box<dyn ConfigurationRoot>, ReloadError>;
}
