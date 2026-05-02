use crate::{Result, Settings};

/// Represents a [configuration provider](Provider) for in-memory data.
#[derive(Debug, Default)]
pub struct Provider {
    /// Gets a list of key/value pairs representing the initial data.
    pub data: Vec<(String, String)>,
}

impl Provider {
    /// Initializes a new in-memory configuration provider.
    ///
    /// # Arguments
    ///
    /// * `data` - The list of key/value pairs representing the initial data
    pub fn new<S: AsRef<str>>(data: &[(S, S)]) -> Self {
        Self {
            data: data
                .iter()
                .map(|t| (t.0.as_ref().to_owned(), t.1.as_ref().to_owned()))
                .collect(),
        }
    }
}

impl crate::Provider for Provider {
    #[inline]
    fn name(&self) -> &str {
        "Memory"
    }

    fn load(&self, settings: &mut Settings) -> Result {
        for (key, value) in &self.data {
            settings.insert(key.clone(), value.clone());
        }

        Ok(())
    }
}
