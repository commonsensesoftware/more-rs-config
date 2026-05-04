use crate::{chained, Builder, Configuration};
use std::sync::Arc;

/// Defines chained configuration extension methods for a [configuration builder](Builder).
pub trait ChainedExt: Sized {
    /// Adds an existing configuration.
    ///
    /// # Arguments
    ///
    /// * `configuration` - The existing [configuration](Configuration) to add
    fn add_configuration<T: Into<Arc<Configuration>>>(self, configuration: T) -> Self;
}

impl ChainedExt for Builder {
    fn add_configuration<T: Into<Arc<Configuration>>>(mut self, configuration: T) -> Self {
        self.add(chained::Provider::new(configuration.into()));
        self
    }
}
