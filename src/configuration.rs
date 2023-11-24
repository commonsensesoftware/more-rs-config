use crate::{ConfigurationPath, ConfigurationSection, Value};
use cfg_if::cfg_if;
use tokens::ChangeToken;

cfg_if! {
    if #[cfg(feature = "async")] {
        /// Defines the behavior of a configuration.
        pub trait Configuration: Send + Sync {
            /// Gets the configuration value.
            ///
            /// # Arguments
            ///
            /// * `key` - The configuration key
            fn get(&self, key: &str) -> Option<Value>;

            /// Gets a [`ConfigurationSection`](crate::ConfigurationSection) with the specified key.
            fn section(&self, key: &str) -> Box<dyn ConfigurationSection>;

            /// Gets the sequence of [`ConfigurationSection`](crate::ConfigurationSection) children.
            fn children(&self) -> Vec<Box<dyn ConfigurationSection>>;

            /// Returns a [`ChangeToken`](tokens::ChangeToken) that can be used to observe when this configuration is reloaded.
            fn reload_token(&self) -> Box<dyn ChangeToken>;

            /// Attempts to convert the [`Configuration`] as a [`ConfigurationSection`](crate::ConfigurationSection).
            fn as_section(&self) -> Option<&dyn ConfigurationSection> {
                None
            }

            /// Gets an iterator of the key/value pairs within the [`Configuration`].
            ///
            /// # Arguments
            ///
            /// * `path` - The type of [`ConfigurationPath`] used when iterating
            fn iter(&self, path: Option<ConfigurationPath>) -> Box<dyn Iterator<Item = (String, Value)>>;
        }
    } else {
        /// Defines the behavior of a configuration.
        pub trait Configuration {
            /// Gets the configuration value.
            ///
            /// # Arguments
            ///
            /// * `key` - The configuration key
            fn get(&self, key: &str) -> Option<Value>;

            /// Gets a [`ConfigurationSection`](crate::ConfigurationSection) with the specified key.
            fn section(&self, key: &str) -> Box<dyn ConfigurationSection>;

            /// Gets the sequence of [`ConfigurationSection`](crate::ConfigurationSection) children.
            fn children(&self) -> Vec<Box<dyn ConfigurationSection>>;

            /// Returns a [`ChangeToken`](tokens::ChangeToken) that can be used to observe when this configuration is reloaded.
            fn reload_token(&self) -> Box<dyn ChangeToken>;

            /// Attempts to convert the [`Configuration`] as a [`ConfigurationSection`](crate::ConfigurationSection).
            fn as_section(&self) -> Option<&dyn ConfigurationSection> {
                None
            }

            /// Gets an iterator of the key/value pairs within the [`Configuration`].
            ///
            /// # Arguments
            ///
            /// * `path` - The type of [`ConfigurationPath`] used when iterating
            fn iter(&self, path: Option<ConfigurationPath>) -> Box<dyn Iterator<Item = (String, Value)>>;
        }
    }
}

/// Represents an iterator of key/value pairs for a [`Configuration`].
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
    /// * `configuration` - The [`Configuration`] to iterate
    /// * `path` - The type of [`ConfigurationPath`] used when iterating
    pub fn new(configuration: &dyn Configuration, path: ConfigurationPath) -> Self {
        let stack = configuration.children();
        let mut first = None;
        let mut prefix_length = 0;

        if let Some(root) = configuration.as_section() {
            if path == ConfigurationPath::Relative {
                prefix_length = root.path().len() + 1;
            } else {
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
