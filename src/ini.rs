use crate::{path, Error, FileSource, Result, Settings};
use configparser::ini::Ini;
use tokens::{ChangeToken, FileChangeToken, NeverChangeToken};

/// Represents a [configuration provider](Provider) for `*.ini` files.
pub struct Provider(FileSource);

impl Provider {
    /// Initializes a new `*.ini` file configuration provider.
    ///
    /// # Arguments
    ///
    /// * `file` - The `*.ini` [file source](FileSource) information
    #[inline]
    pub fn new(file: FileSource) -> Self {
        Self(file)
    }
}

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
