use crate::{ConfigurationSection, Value};
use tokens::ChangeToken;

/// Defines the behavior of a configuration.
pub trait Configuration {
    /// Gets the configuration value.
    ///
    /// # Arguments
    ///
    /// * `key` - The configuration key
    fn get(&self, key: &str) -> Option<Value>;

    /// Gets a [configuration section](trait.ConfigurationSection.html) with the specified key.
    fn section(&self, key: &str) -> Box<dyn ConfigurationSection>;

    /// Gets the sequence of child [configuration sections](trait.ConfigurationSection.html).
    fn children(&self) -> Vec<Box<dyn ConfigurationSection>>;

    /// Returns a change token that can be used to observe when this configuration is reloaded.
    fn reload_token(&self) -> Box<dyn ChangeToken>;

    /// Attempts to convert the [configuration](trait.Configuration.html) as a [configuration section](trait.ConfigurationSection.html).
    fn as_section(&self) -> Option<&dyn ConfigurationSection> {
        None
    }

    /// Gets an iterator of the key/value pairs within the [configuration](trait.Configuration.html).
    fn iter(&self) -> Box<dyn Iterator<Item = (String, Value)>> {
        self.iter_relative(false)
    }

    /// Gets an iterator of the key/value pairs within the [configuration](trait.Configuration.html).
    ///
    /// # Arguments
    ///
    /// * `make_paths_relative` - If true, the child keys returned will have the current configuration's path trimmed from the front
    fn iter_relative(
        &self,
        make_paths_relative: bool,
    ) -> Box<dyn Iterator<Item = (String, Value)>>;
}

/// Represents an iterator of key/value pairs for a [configuration](trait.Configuration.html).
pub struct ConfigurationIterator {
    stack: Vec<Box<dyn ConfigurationSection>>,
    first: Option<(String, Value)>,
    prefix_length: usize,
}

impl ConfigurationIterator {
    /// Initializes a new configuration iterator.
    ///
    /// # Arguments
    ///
    /// * `configuration` - The [configuration](trait.Configuration.html) to iterate
    /// * `make_paths_relative` - If true, the child keys returned will have the current configuration's path trimmed from the front
    pub fn new(configuration: &dyn Configuration, make_paths_relative: bool) -> Self {
        let stack = configuration.children();
        let mut first = None;
        let mut prefix_length = 0;

        if let Some(root) = configuration.as_section() {
            if make_paths_relative {
                prefix_length = root.path().len() + 1;
            }

            if !make_paths_relative {
                let key = root.path()[prefix_length..].to_owned();
                let value = root.value();

                first = Some((key, value));
            }
        }

        Self {
            stack,
            first,
            prefix_length,
        }
    }
}

impl Iterator for ConfigurationIterator {
    type Item = (String, Value);

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(first) = self.first.take() {
            return Some(first);
        }

        while let Some(config) = self.stack.pop() {
            self.stack.extend(config.children().into_iter());

            if let Some(section) = config.as_section() {
                let key = section.path()[self.prefix_length..].to_owned();
                let value = section.value();
                return Some((key, value));
            }
        }

        None
    }
}
