use crate::{Properties, Root, Source};

/// Represents a [configuration](crate::Configuration) builder.
#[derive(Default)]
pub struct Builder {
    sources: Vec<Box<dyn Source>>,

    /// Gets or sets properties that can be passed to [configuration sources](Source).
    pub properties: Properties,
}

impl Builder {
    /// Gets the registered [sources](Source) used to obtain configuration values.
    #[inline]
    pub fn sources(&self) -> &[Box<dyn Source>] {
        &self.sources
    }

    /// Adds a new configuration source.
    ///
    /// # Arguments
    ///
    /// * `source` - The [configuration source](Source) to add
    #[inline]
    pub fn add(&mut self, source: impl Source + 'static) {
        self.sources.push(Box::new(source))
    }

    /// Builds a [configuration root](Root) from the registered [configuration sources](Source).
    pub fn build(mut self) -> Root {
        let mut properties = self.properties;
        Root::new(self.sources.iter_mut().map(|s| s.build(&mut properties)))
    }
}
