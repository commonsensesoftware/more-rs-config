use crate::{ConfigurationProvider, ConfigurationBuilder};

/// Represents a source of configuration key/value pairs for an application.
pub trait ConfigurationSource {
    /// Builds the [configuration provider](trait.ConfigurationProvider.html) for this source.
    /// 
    /// # Arguments
    /// 
    /// * `builder` - The [builder](trait.ConfigurationBuilder.html) used to build the provider
    fn build(&self, builder: &dyn ConfigurationBuilder) -> Box<dyn ConfigurationProvider>;
}