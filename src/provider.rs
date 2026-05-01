use crate::{Result, Settings};
use tokens::{ChangeToken, NeverChangeToken};

/// Defines the behavior of an object that provides configuration settings.
#[cfg_attr(feature = "async", maybe_impl::traits(Send, Sync))]
pub trait Provider {
    /// Gets the name of the provider.
    fn name(&self) -> &str;

    /// Returns a [change token](ChangeToken) that indicates when this provider has changed.
    fn reload_token(&self) -> Box<dyn ChangeToken> {
        Box::new(NeverChangeToken)
    }

    /// Loads the provided configuration.
    ///
    /// # Arguments
    ///
    /// * `settings` - The [settings](Settings) to load the configured values into
    fn load(&self, settings: &mut Settings) -> Result;
}
