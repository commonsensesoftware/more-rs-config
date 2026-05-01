use crate::{Properties, Provider};

/// Represents a source for provided configuration settings.
pub trait Source {
    /// Builds the [configuration provider](Provider) for this source.
    ///
    /// # Arguments
    ///
    /// * `properties` - The [properties](Properties) used to build the provider
    fn build(&mut self, properties: &mut Properties) -> Box<dyn Provider>;
}
