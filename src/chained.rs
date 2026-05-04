use crate::{Configuration, Merge, Result, Settings};
use std::sync::Arc;
use tokens::ChangeToken;

/// Represents a chained [configuration provider](crate::Provider).
pub struct Provider(Arc<Configuration>);

impl Provider {
    /// Initializes a new chained configuration provider.
    ///
    /// # Arguments
    ///
    /// * `configuration` - The [configuration](Configuration) to chain
    #[inline]
    pub fn new(configuration: Arc<Configuration>) -> Self {
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
        Box::new(self.0.change_token())
    }

    fn load(&self, settings: &mut Settings) -> Result {
        settings.merge(&*self.0);
        Ok(())
    }
}
