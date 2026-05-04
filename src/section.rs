use crate::{path, Configuration, Settings};
use tokens::{ChangeToken, NeverChangeToken};

macro_rules! section {
    ($self:ident) => {
        /// Gets the key of the section.
        #[inline]
        pub fn key(&$self) -> &str {
            path::last(&$self.path)
        }

        /// Gets the path of the section.
        #[inline]
        pub fn path(&$self) -> &str {
            &$self.path
        }

        /// Gets the value of the section, if any.
        #[inline]
        pub fn value(&$self) -> &str {
            $self.config.settings.get(&$self.path).unwrap_or_default()
        }

        /// Determines if the section exists.
        ///
        /// # Remarks
        ///
        /// A section exists if either its value or its subsections are not empty.
        pub fn exists(&$self) -> bool {
            !$self.value().is_empty() || !$self.sections().is_empty()
        }

        /// Gets a configuration value in this section.
        ///
        /// # Arguments
        ///
        /// * `key` - The case-insensitive key of the configuration value to get
        #[inline]
        pub fn get(&$self, key: &str) -> Option<&str> {
            $self.config.settings.get_subkey(&$self.path, key)
        }
    };
}

fn collect_section_keys(config: &Configuration, parent: &str) -> Vec<String> {
    let mut keys = Vec::new();

    for (path, _) in config {
        if let Some(key) = path::next(path, Some(parent)) {
            if !keys.iter().any(|k: &String| k.eq_ignore_ascii_case(key)) {
                keys.push(key.to_owned());
            }
        }
    }

    keys.sort_by(path::cmp);
    keys
}

/// Represents a [configuration](Configuration) section.
#[derive(Clone)]
pub struct Section<'a> {
    config: &'a Configuration,
    path: String,
}

impl<'a> Section<'a> {
    #[inline]
    pub(crate) fn new(config: &'a Configuration, path: String) -> Self {
        Self { config, path }
    }

    /// Gets a configuration [subsection](Section) in this section.
    ///
    /// # Arguments
    ///
    /// * `key` - The case-insensitive key of the configuration subsection to get
    #[inline]
    pub fn section(&self, key: &str) -> Section<'a> {
        self.config.section(path::combine(&[&self.path, key]))
    }

    /// Gets all of the [subsections](Section) in this section.
    pub fn sections(&self) -> Vec<Section<'a>> {
        collect_section_keys(self.config, &self.path)
            .into_iter()
            .map(|key| self.config.section(key))
            .collect()
    }

    /// Gets an owned copy of the section.
    ///
    /// # Remarks
    ///
    /// This function is useful for taking ownership of a section in order to decouple it from the entire
    /// [configuration](Configuration) that created it. The owned section holds a reference to the subset of key/value
    /// pairs at this point in the [configuration](Configuration).
    #[inline]
    pub fn to_owned(&self) -> OwnedSection {
        let len = self.path.len();
        let token: Box<dyn ChangeToken> = Box::new(NeverChangeToken);
        let mut settings = Settings::new();

        for (key, value) in self.config {
            if key.len() > len && path::starts_with(key, &self.path) {
                settings.insert(key, value);
            }
        }

        OwnedSection {
            config: Configuration::new(settings, [token]),
            path: self.path.clone(),
        }
    }

    section!(self);
}

impl<'a> From<Section<'a>> for Vec<Section<'a>> {
    #[inline]
    fn from(section: Section<'a>) -> Self {
        section.sections()
    }
}

impl<'a> From<&'a Section<'a>> for Vec<Section<'a>> {
    #[inline]
    fn from(section: &'a Section<'a>) -> Self {
        section.sections()
    }
}

/// Represents an owned [configuration section](Section).
#[derive(Clone)]
pub struct OwnedSection {
    config: Configuration,
    path: String,
}

impl OwnedSection {
    /// Gets a configuration [subsection](Section) in this section.
    ///
    /// # Arguments
    ///
    /// * `key` - The case-insensitive key of the configuration subsection to get
    #[inline]
    pub fn section(&self, key: &str) -> Section<'_> {
        self.config.section(path::combine(&[&self.path, key]))
    }

    /// Gets all of the [subsections](Section) in this section.
    pub fn sections(&self) -> Vec<Section<'_>> {
        collect_section_keys(&self.config, &self.path)
            .into_iter()
            .map(|key| self.config.section(key))
            .collect()
    }

    section!(self);
}
