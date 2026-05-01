use crate::{path, settings, Section, Settings};
use tokens::{ChangeToken, CompositeChangeToken, SharedChangeToken};

/// Represents a configuration.
pub struct Configuration {
    pub(crate) settings: Settings,
    token: SharedChangeToken<CompositeChangeToken>,
}

impl Configuration {
    /// Initializes a new [Configuration].
    ///
    /// # Arguments
    ///
    /// * `settings` - The configuration [settings](Settings)
    /// * `tokens` - The [sequence](Iterator) of [change tokens](ChangeToken) associated with the configuration
    #[inline]
    pub fn new(settings: Settings, tokens: impl IntoIterator<Item = Box<dyn ChangeToken>>) -> Self {
        Self {
            settings,
            token: SharedChangeToken::new(CompositeChangeToken::new(tokens.into_iter())),
        }
    }

    /// Gets a configuration value.
    ///
    /// # Arguments
    ///
    /// * `key` - The case-insensitive key of the configuration value to get
    #[inline]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.settings.get(key)
    }

    /// Gets a section in this configuration.
    ///
    /// # Arguments
    ///
    /// * `key` - The case-insensitive key of the configuration subsection to get
    #[inline]
    pub fn section(&self, key: impl Into<String>) -> Section<'_> {
        Section::new(self, key.into())
    }

    /// Gets all of the sections in this configuration.
    pub fn sections(&self) -> Vec<Section<'_>> {
        let mut keys = Vec::new();

        for (path, _) in self {
            if let Some(key) = path::next(path, None) {
                if !keys.iter().any(|k: &String| k.eq_ignore_ascii_case(key)) {
                    keys.push(key.to_owned());
                }
            }
        }

        keys.sort_by(path::cmp);
        keys.into_iter().map(|key| self.section(key)).collect()
    }

    /// Returns a [change token](ChangeToken) that indicates when the configuration has changed.
    #[inline]
    pub fn reload_token(&self) -> impl ChangeToken {
        self.token.clone()
    }
}

impl<'a> IntoIterator for &'a Configuration {
    type Item = (&'a str, &'a str);
    type IntoIter = settings::Iter<'a>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        (&self.settings).into_iter()
    }
}

impl IntoIterator for Configuration {
    type Item = (String, String);
    type IntoIter = settings::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.settings.into_iter()
    }
}

impl<'a> From<&'a Configuration> for Vec<Section<'a>> {
    #[inline]
    fn from(config: &'a Configuration) -> Self {
        config.sections()
    }
}
