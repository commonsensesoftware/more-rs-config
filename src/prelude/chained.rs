use crate::{chained, Builder, Configuration, Ref};

/// Defines chained configuration extension methods for a [configuration builder](Builder).
pub trait ChainedExt: Sized {
    /// Adds an existing configuration.
    ///
    /// # Arguments
    ///
    /// * `configuration` - The existing [configuration](Configuration) to add
    fn add_configuration<T: Into<Ref<Configuration>>>(self, configuration: T) -> Self;
}

impl ChainedExt for Builder {
    fn add_configuration<T: Into<Ref<Configuration>>>(mut self, configuration: T) -> Self {
        self.add(chained::Source::new(configuration.into()));
        self
    }
}
