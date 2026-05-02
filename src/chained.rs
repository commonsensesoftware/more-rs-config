use crate::{Configuration, Merge, Ref, Result, Settings};
use tokens::ChangeToken;

/// Represents a chained [configuration provider](Provider).
pub struct Provider(Ref<Configuration>);

impl Provider {
    /// Initializes a new chained configuration provider.
    ///
    /// # Arguments
    ///
    /// * `configuration` - The [configuration](Configuration) to chain
    #[inline]
    pub fn new(configuration: Ref<Configuration>) -> Self {
        Self(configuration)
    }
}

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
