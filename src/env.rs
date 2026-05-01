use crate::{Properties, Result, Settings};
use std::{env::vars, mem::take};

fn escape(name: String) -> String {
    if name.contains("__") {
        name.replace("__", ":")
    } else {
        name
    }
}

fn starts_with(text: &str, other: &str) -> bool {
    text.len() >= other.len() && text.chars().zip(other.chars()).all(|(l, r)| l.eq_ignore_ascii_case(&r))
}

#[derive(Debug)]
struct Provider(String);

impl crate::Provider for Provider {
    #[inline]
    fn name(&self) -> &str {
        "Environment"
    }

    fn load(&self, settings: &mut Settings) -> Result {
        if self.0.is_empty() {
            for (key, value) in vars() {
                settings.insert(escape(key), value);
            }
        } else {
            let prefix = &self.0;
            let len = prefix.len();

            for (key, value) in vars() {
                if starts_with(&key, prefix) {
                    settings.insert(escape(key[len..].to_string()), value);
                }
            }
        }

        Ok(())
    }
}

/// Represents a [configuration source](Source) for environment variables.
#[derive(Default)]
pub struct Source {
    /// A prefix used to filter environment variables.
    pub prefix: String,
}

impl Source {
    /// Initializes a new environment variables configuration source.
    ///
    /// # Arguments
    ///
    /// * `prefix` - A prefix used to filter environment variables
    #[inline]
    pub fn new(prefix: impl Into<String>) -> Self {
        Self { prefix: prefix.into() }
    }
}

impl crate::Source for Source {
    #[inline]
    fn build(&mut self, _properties: &mut Properties) -> Box<dyn crate::Provider> {
        Box::new(Provider(take(&mut self.prefix)))
    }
}
