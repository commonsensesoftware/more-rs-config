use crate::{path, Configuration};

/// Represents a [configuration](Configuration) section.
#[derive(Clone)]
pub struct Section<'a> {
    cfg: &'a Configuration,
    path: String,
}

impl<'a> Section<'a> {
    #[inline]
    pub(crate) fn new(cfg: &'a Configuration, path: String) -> Self {
        Self { cfg, path }
    }

    /// Gets a configuration [subsection](Section) in this section.
    ///
    /// # Arguments
    ///
    /// * `key` - The case-insensitive key of the configuration subsection to get
    #[inline]
    pub fn section(&self, key: &str) -> Section<'a> {
        self.cfg.section(&path::combine(&[&self.path, key]))
    }

    /// Gets all of the [subsections](Section) in this section.
    pub fn sections(&self) -> Vec<Section<'a>> {
        let mut keys = Vec::new();

        for (path, _) in self.cfg {
            if let Some(key) = path::next(path, Some(&self.path)) {
                if !keys.iter().any(|k: &String| k.eq_ignore_ascii_case(key)) {
                    keys.push(key.to_owned());
                }
            }
        }

        keys.sort_by(path::cmp);
        keys.into_iter().map(|key| self.cfg.section(key)).collect()
    }
}

impl Section<'_> {
    /// Gets the key of the section.
    #[inline]
    pub fn key(&self) -> &str {
        path::last(&self.path)
    }

    /// Gets the path of the section.
    #[inline]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Gets the value of the section, if any.
    #[inline]
    pub fn value(&self) -> &str {
        self.cfg.settings.get(&self.path).unwrap_or_default()
    }

    /// Determines if the section exists.
    ///
    /// # Remarks
    ///
    /// A section exists if either its value or its subsections are not empty.
    pub fn exists(&self) -> bool {
        !self.value().is_empty() || !self.sections().is_empty()
    }

    /// Gets a configuration value in this section.
    ///
    /// # Arguments
    ///
    /// * `key` - The case-insensitive key of the configuration value to get
    #[inline]
    pub fn get(&self, key: &str) -> Option<&str> {
        self.cfg.settings.get_subkey(&self.path, key)
    }
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
