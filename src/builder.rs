use crate::{Configuration, Provider, Result, Settings};

/// Represents a [configuration](crate::Configuration) builder.
#[derive(Default)]
pub struct Builder(Vec<Box<dyn Provider>>);

impl Builder {
    /// Gets the [configuration providers](Provider).
    #[inline]
    pub fn providers(&self) -> impl Iterator<Item = &dyn Provider> {
        self.0.iter().map(AsRef::as_ref)
    }

    /// Adds a new configuration provider.
    ///
    /// # Arguments
    ///
    /// * `provider` - The [configuration provider](Provider) to add
    #[inline]
    pub fn add(&mut self, provider: impl Provider + 'static) {
        self.0.push(Box::new(provider))
    }

    /// Builds a [configuration](Configuration) from the registered [configuration providers](Provider).
    pub fn build(&self) -> Result<Configuration> {
        let mut settings = Settings::new();
        let mut tokens = Vec::with_capacity(self.0.len());

        for provider in &self.0 {
            provider.load(&mut settings)?;
            tokens.push(provider.reload_token());
        }

        settings.shrink_to_fit();

        Ok(Configuration::new(settings, tokens))
    }
}
