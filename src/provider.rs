use std::{any::type_name, borrow::Cow};
use tokens::{ChangeToken, NeverChangeToken};

/// Defines the behavior of an object that provides configuration key/values for an application.
pub trait ConfigurationProvider {
    /// Gets the name of the provider.
    fn name(&self) -> &str {
        type_name::<Self>()
    }

    /// Attempts to get a configuration value with the specified key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the value to retrieve
    fn get(&self, key: &str) -> Option<Cow<String>>;

    /// Returns a change token if this provider supports change tracking.
    fn reload_token(&self) -> Box<dyn ChangeToken> {
        Box::new(NeverChangeToken::new())
    }

    /// Loads the configuration values from the implemented source.
    fn load(&mut self) {}

    /// Gets the immediate descendent configuration keys for a given parent path based on this
    /// [provider](trait.ConfigurationProvider.html) and the set of keys returned by all of
    /// the preceding [providers](trait.ConfigurationProvider.html).
    ///
    /// # Arguments
    ///
    /// * `earlier_keys` - The sequence of keys returned by preceding provider for the same parent path
    /// * `parent_path` - The optional parent path to evaluate
    fn child_keys(&self, earlier_keys: &mut Vec<String>, parent_path: Option<&str>);
}
