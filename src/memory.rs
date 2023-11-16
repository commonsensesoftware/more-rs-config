use crate::{
    util::accumulate_child_keys, ConfigurationBuilder, ConfigurationProvider, ConfigurationSource,
};
use std::{borrow::Cow, collections::HashMap};

/// Represents a [configuration provider](trait.ConfigurationProvider.html) that
/// provides in-memory configuration values.
pub struct MemoryConfigurationProvider {
    data: HashMap<String, (String, String)>,
}

impl MemoryConfigurationProvider {
    /// Initializes a new in-memory configuration provider.
    ///
    /// # Arguments
    ///
    /// * `data` - The in-memory data associated with the provider
    ///
    /// # Remarks
    ///
    /// The data key is normalized to uppercase. The value is a tuple where the
    /// first item is the originally-cased key and the second item is value.
    pub fn new(data: HashMap<String, (String, String)>) -> Self {
        Self { data }
    }
}

impl ConfigurationProvider for MemoryConfigurationProvider {
    fn get(&self, key: &str) -> Option<Cow<String>> {
        self.data
            .get(&key.to_uppercase())
            .map(|t| Cow::Borrowed(&t.1))
    }

    fn child_keys(&self, earlier_keys: &mut Vec<String>, parent_path: Option<&str>) {
        accumulate_child_keys(&self.data, earlier_keys, parent_path)
    }
}

/// Represents a [configuration source](trait.ConfigurationSource.html) for in-memory data.
#[derive(Default)]
pub struct MemoryConfigurationSource {
    /// Gets a list of key/value pairs representing the initial data.
    pub initial_data: Vec<(String, String)>,
}

impl MemoryConfigurationSource {
    /// Initializes a new in-memory configuration source.
    ///
    /// # Arguments
    ///
    /// * `initial_data` - The list of key/value pairs representing the initial data
    pub fn new<S: AsRef<str>>(initial_data: &[(S, S)]) -> Self {
        Self {
            initial_data: initial_data
                .iter()
                .map(|t| (t.0.as_ref().to_owned(), t.1.as_ref().to_owned()))
                .collect(),
        }
    }
}

impl ConfigurationSource for MemoryConfigurationSource {
    fn build(&self, _builder: &dyn ConfigurationBuilder) -> Box<dyn ConfigurationProvider> {
        let data: HashMap<_, _> = self
            .initial_data
            .iter()
            .map(|t| (t.0.to_uppercase(), (t.0.clone(), t.1.clone())))
            .collect();
        Box::new(MemoryConfigurationProvider::new(data))
    }
}

pub mod ext {

    use super::*;

    /// Defines extension methods for the [ConfigurationBuilder](trait.ConfigurationBuilder.html) trait.
    pub trait MemoryConfigurationBuilderExtensions {
        /// Adds the in-memory configuration source using the specified data.
        ///
        /// # Arguments
        ///
        /// * `data` - The data to add to memory configuration provider
        fn add_in_memory<S: AsRef<str>>(&mut self, data: &[(S, S)]) -> &mut Self;
    }

    impl MemoryConfigurationBuilderExtensions for dyn ConfigurationBuilder {
        fn add_in_memory<S: AsRef<str>>(&mut self, data: &[(S, S)]) -> &mut Self {
            self.add(Box::new(MemoryConfigurationSource::new(data)));
            self
        }
    }

    impl<T: ConfigurationBuilder> MemoryConfigurationBuilderExtensions for T {
        fn add_in_memory<S: AsRef<str>>(&mut self, data: &[(S, S)]) -> &mut Self {
            self.add(Box::new(MemoryConfigurationSource::new(data)));
            self
        }
    }
}
