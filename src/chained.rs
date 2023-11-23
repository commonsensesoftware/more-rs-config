use tokens::ChangeToken;

use crate::{
    util::cmp_keys, Configuration, ConfigurationBuilder, ConfigurationProvider,
    ConfigurationSource, Value,
};
use std::borrow::Borrow;
use std::rc::Rc;

/// Represents a chained [`ConfigurationProvider`](crate::ConfigurationProvider).
pub struct ChainedConfigurationProvider {
    configuration: Rc<dyn Configuration>,
}

impl ChainedConfigurationProvider {
    /// Initializes a new chained configuration provider.
    ///
    /// # Arguments
    ///
    /// * `configuration` - The [`Configuration`](crate::Configuration) to chain
    pub fn new(configuration: Rc<dyn Configuration>) -> Self {
        Self { configuration }
    }
}

impl ConfigurationProvider for ChainedConfigurationProvider {
    fn get(&self, key: &str) -> Option<Value> {
        self.configuration.get(key)
    }

    fn reload_token(&self) -> Box<dyn ChangeToken> {
        self.configuration.reload_token()
    }

    fn child_keys(&self, earlier_keys: &mut Vec<String>, parent_path: Option<&str>) {
        if let Some(path) = parent_path {
            earlier_keys.extend(
                self.configuration
                    .section(path)
                    .children()
                    .iter()
                    .map(|c| c.key().to_owned()),
            );
        } else {
            earlier_keys.extend(
                self.configuration
                    .children()
                    .iter()
                    .map(|c| c.key().to_owned()),
            );
        }

        earlier_keys.sort_by(|k1, k2| cmp_keys(k1, k2));
    }
}

/// Represents a chained [`ConfigurationSource`](crate::ConfigurationSource).
pub struct ChainedConfigurationSource {
    configuration: Rc<dyn Configuration>,
}

impl ChainedConfigurationSource {
    /// Initializes a new chained configuration sources.
    ///
    /// # Arguments
    ///
    /// * `configuration` - The [`Configuration`](crate::Configuration) to chain
    pub fn new(configuration: Box<dyn Configuration>) -> Self {
        Self {
            configuration: Rc::from(configuration),
        }
    }

    /// Gets the associated [`Configuration`](crate::Configuration).
    pub fn configuration(&self) -> &dyn Configuration {
        self.configuration.borrow()
    }
}

impl ConfigurationSource for ChainedConfigurationSource {
    fn build(&self, _builder: &dyn ConfigurationBuilder) -> Box<dyn ConfigurationProvider> {
        Box::new(ChainedConfigurationProvider::new(
            self.configuration.clone(),
        ))
    }
}

pub mod ext {

    use super::*;

    /// Defines extension methods for [`ConfigurationBuilder`](crate::ConfigurationBuilder).
    pub trait ChainedBuilderExtensions {
        /// Adds the existing configuration.
        ///
        /// # Arguments
        ///
        /// * `configuration` - The existing [`Configuration`](crate::Configuration) to add
        fn add_configuration(&mut self, configuration: Box<dyn Configuration>) -> &mut Self;
    }

    impl ChainedBuilderExtensions for dyn ConfigurationBuilder {
        fn add_configuration(&mut self, configuration: Box<dyn Configuration>) -> &mut Self {
            self.add(Box::new(ChainedConfigurationSource::new(configuration)));
            self
        }
    }

    impl<T: ConfigurationBuilder> ChainedBuilderExtensions for T {
        fn add_configuration(&mut self, configuration: Box<dyn Configuration>) -> &mut Self {
            self.add(Box::new(ChainedConfigurationSource::new(configuration)));
            self
        }
    }
}
