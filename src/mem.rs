use crate::{properties::Properties, Result, Settings};
use std::mem::take;

#[derive(Debug)]
struct Provider(Vec<(String, String)>);

impl crate::Provider for Provider {
    #[inline]
    fn name(&self) -> &str {
        "Memory"
    }

    fn load(&self, settings: &mut Settings) -> Result {
        for (key, value) in &self.0 {
            settings.insert(key.clone(), value.clone());
        }

        Ok(())
    }
}

/// Represents a [configuration source](Source) for in-memory data.
#[derive(Default)]
pub struct Source {
    /// Gets a list of key/value pairs representing the initial data.
    pub data: Vec<(String, String)>,
}

impl Source {
    /// Initializes a new in-memory configuration source.
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

impl crate::Source for Source {
    #[inline]
    fn build(&mut self, _properties: &mut Properties) -> Box<dyn crate::Provider> {
        Box::new(Provider(take(&mut self.data)))
    }
}
