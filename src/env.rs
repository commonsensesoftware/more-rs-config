use crate::{
    util::accumulate_child_keys, ConfigurationBuilder, ConfigurationProvider, ConfigurationSource,
    LoadResult,
};
use std::collections::HashMap;
use std::env::vars;

/// Represents a [configuration provider](trait.ConfigurationProvider.html) for environment variables.
#[derive(Default)]
pub struct EnvironmentVariablesConfigurationProvider {
    prefix: String,
    data: HashMap<String, (String, String)>,
}

impl EnvironmentVariablesConfigurationProvider {
    /// Initializes a new environment variables configuration provider.
    ///
    /// # Arguments
    ///
    /// * `prefix` - A prefix used to filter the environment variables
    pub fn new(prefix: String) -> Self {
        Self {
            prefix,
            data: HashMap::with_capacity(0),
        }
    }
}

impl ConfigurationProvider for EnvironmentVariablesConfigurationProvider {
    fn get(&self, key: &str) -> Option<String> {
        self.data
            .get(&key.to_uppercase())
            .map(|t| t.1.clone())
    }

    fn load(&mut self) -> LoadResult {
        let mut data = HashMap::new();
        let prefix = self.prefix.to_uppercase();
        let prefix_len = self.prefix.len();

        for (key, value) in vars() {
            if key.to_uppercase().starts_with(&prefix) {
                let new_key = key[prefix_len..].to_string();
                data.insert(new_key.to_uppercase().replace("__", ":"), (new_key, value));
            }
        }

        self.data = data;
        Ok(())
    }

    fn child_keys(&self, earlier_keys: &mut Vec<String>, parent_path: Option<&str>) {
        accumulate_child_keys(&self.data, earlier_keys, parent_path)
    }
}

/// Represents a [configuration source](trait.ConfigurationSource.html) for environment variables.
#[derive(Default)]
pub struct EnvironmentVariablesConfigurationSource {
    /// A prefix used to filter environment variables.
    pub prefix: String,
}

impl EnvironmentVariablesConfigurationSource {
    /// Initializes a new environment variables configuration source.
    ///
    /// # Arguments
    ///
    /// * `prefix` - A prefix used to filter environment variables
    pub fn new(prefix: &str) -> Self {
        Self {
            prefix: prefix.to_owned(),
        }
    }
}

impl ConfigurationSource for EnvironmentVariablesConfigurationSource {
    fn build(&self, _builder: &dyn ConfigurationBuilder) -> Box<dyn ConfigurationProvider> {
        Box::new(EnvironmentVariablesConfigurationProvider::new(
            self.prefix.clone(),
        ))
    }
}

pub mod ext {

    use super::*;

    /// Defines extension methods for the [ConfigurationBuilder](trait.ConfigurationBuilder.html) trait.
    pub trait EnvironmentVariablesExtensions {
        /// Adds environment variables as a configuration source.
        fn add_env_vars(&mut self) -> &mut Self;

        /// Adds environment variables as a configuration source.
        ///
        /// # Arguments
        ///
        /// * `prefix` - The prefix that environment variable names must start with.
        ///              The prefix will be removed from the environment variable names.
        fn add_env_vars_with_prefix(&mut self, prefix: &str) -> &mut Self;
    }

    impl EnvironmentVariablesExtensions for dyn ConfigurationBuilder {
        fn add_env_vars(&mut self) -> &mut Self {
            self.add_env_vars_with_prefix("")
        }

        fn add_env_vars_with_prefix(&mut self, prefix: &str) -> &mut Self {
            self.add(Box::new(EnvironmentVariablesConfigurationSource::new(
                prefix,
            )));
            self
        }
    }

    impl<T: ConfigurationBuilder> EnvironmentVariablesExtensions for T {
        fn add_env_vars(&mut self) -> &mut Self {
            self.add_env_vars_with_prefix("")
        }

        fn add_env_vars_with_prefix(&mut self, prefix: &str) -> &mut Self {
            self.add(Box::new(EnvironmentVariablesConfigurationSource::new(
                prefix,
            )));
            self
        }
    }
}
