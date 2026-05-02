use crate::{pascal_case, Result, Settings};
use std::{borrow::Cow, env::vars};

fn escape(name: &str) -> Cow<'_, str> {
    if name.contains("__") {
        Cow::Owned(name.replace("__", ":"))
    } else {
        Cow::Borrowed(name)
    }
}

fn starts_with(text: &str, other: &str) -> bool {
    text.len() >= other.len() && text.chars().zip(other.chars()).all(|(l, r)| l.eq_ignore_ascii_case(&r))
}

/// Represents a [configuration provider](crate::Provider) for environment variables.
#[derive(Debug, Default)]
pub struct Provider(String);

impl Provider {
    /// Initializes a new environment variables configuration provider.
    ///
    /// # Arguments
    ///
    /// * `prefix` - A prefix used to filter environment variables
    #[inline]
    pub fn new(prefix: impl Into<String>) -> Self {
        Self(prefix.into())
    }
}

impl crate::Provider for Provider {
    #[inline]
    fn name(&self) -> &str {
        "Environment"
    }

    fn load(&self, settings: &mut Settings) -> Result {
        if self.0.is_empty() {
            for (key, value) in vars() {
                settings.insert(pascal_case(&escape(&key)), value);
            }
        } else {
            let prefix = &self.0;
            let len = prefix.len();

            for (key, value) in vars() {
                if starts_with(&key, prefix) {
                    settings.insert(pascal_case(&escape(&key[len..])), value);
                }
            }
        }

        Ok(())
    }
}
