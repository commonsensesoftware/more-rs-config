use crate::{Configuration, Provider, Result, Settings};

/// Represents the root of a configuration hierarchy.
pub struct Root(Vec<Box<dyn Provider>>);

impl Root {
    /// Initializes a new [Root] configuration.
    ///
    /// # Arguments
    ///
    /// * `providers` - The [sequence](IntoIterator) of [providers](Provider)
    #[inline]
    pub fn new(providers: impl IntoIterator<Item = Box<dyn Provider>>) -> Self {
        Self(providers.into_iter().collect())
    }

    /// Gets the [configuration providers](Provider).
    #[inline]
    pub fn providers(&self) -> impl Iterator<Item = &dyn Provider> {
        self.0.iter().map(AsRef::as_ref)
    }

    /// Load the values from the underlying [providers](Provider) into a [configuration](Configuration).
    pub fn load(&self) -> Result<Configuration> {
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
