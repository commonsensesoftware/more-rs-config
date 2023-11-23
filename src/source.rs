use crate::{ConfigurationProvider, ConfigurationBuilder};

/// Represents a source of configuration key/value pairs for an application.
pub trait ConfigurationSource {
    /// Builds the [`ConfigurationProvider`](crate::ConfigurationProvider) for this source.
    /// 
    /// # Arguments
    /// 
    /// * `builder` - The [`ConfigurationBuilder`](crate::ConfigurationBuilder) used to build the provider
    fn build(&self, builder: &dyn ConfigurationBuilder) -> Box<dyn ConfigurationProvider>;
}