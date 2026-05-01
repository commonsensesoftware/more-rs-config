use crate::{properties::Properties, Configuration, Merge, Ref, Result, Settings};
use tokens::ChangeToken;

struct Provider(Ref<Configuration>);

impl crate::Provider for Provider {
    #[inline]
    fn name(&self) -> &str {
        "Chained"
    }

    #[inline]
    fn reload_token(&self) -> Box<dyn ChangeToken> {
        Box::new(self.0.reload_token())
    }

    fn load(&self, settings: &mut Settings) -> Result {
        settings.merge(&*self.0);
        Ok(())
    }
}

/// Represents a chained [configuration source](Source).
pub struct Source(Ref<Configuration>);

impl Source {
    /// Initializes a new chained configuration source.
    ///
    /// # Arguments
    ///
    /// * `configuration` - The [configuration](Configuration) to chain
    #[inline]
    pub fn new(configuration: Ref<Configuration>) -> Self {
        Self(configuration)
    }
}

impl crate::Source for Source {
    #[inline]
    fn build(&mut self, _properties: &mut Properties) -> Box<dyn crate::Provider> {
        Box::new(Provider(self.0.clone()))
    }
}
