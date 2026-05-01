use crate::{path, properties::Properties, Error, FileSource, Result, Settings};
use configparser::ini::Ini;
use std::mem::take;
use tokens::{ChangeToken, FileChangeToken, NeverChangeToken};

struct Provider(FileSource);

impl crate::Provider for Provider {
    #[inline]
    fn name(&self) -> &str {
        "Ini"
    }

    fn reload_token(&self) -> Box<dyn ChangeToken> {
        if self.0.reload_on_change {
            Box::new(FileChangeToken::new(self.0.path.clone()))
        } else {
            Box::new(NeverChangeToken)
        }
    }

    fn load(&self, settings: &mut Settings) -> Result {
        if !self.0.path.is_file() {
            if self.0.optional {
                return Ok(());
            } else {
                return Err(Error::MissingFile(self.0.path.clone()));
            }
        }

        let mut ini = Ini::new_cs();
        let sections = ini.load(&self.0.path).map_err(Error::Custom)?;

        for (section, pairs) in sections {
            for (key, value) in pairs {
                let key = format!("{section}{}{key}", path::delimiter());
                settings.insert(key, value.unwrap_or_default());
            }
        }

        Ok(())
    }
}

/// Represents a [configuration source](Source) for `*.ini` files.
pub struct Source(FileSource);

impl Source {
    /// Initializes a new `*.ini` file configuration source.
    ///
    /// # Arguments
    ///
    /// * `file` - The `*.ini` [file source](FileSource) information
    #[inline]
    pub fn new(file: FileSource) -> Self {
        Self(file)
    }
}

impl crate::Source for Source {
    #[inline]
    fn build(&mut self, _properties: &mut Properties) -> Box<dyn crate::Provider> {
        Box::new(Provider(take(&mut self.0)))
    }
}
