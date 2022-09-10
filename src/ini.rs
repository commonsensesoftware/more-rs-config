use crate::{
    util::accumulate_child_keys, ConfigurationBuilder, ConfigurationPath, ConfigurationProvider,
    ConfigurationSource,
};
use configparser::ini::Ini;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Represents a [configuration provider](trait.ConfigurationProvider.html) for INI files.
pub struct IniConfigurationProvider {
    path: PathBuf,
    optional: bool,
    data: HashMap<String, (String, String)>,
}

impl IniConfigurationProvider {
    /// Initializes a new INI file configuration provider.
    ///
    /// # Arguments
    ///
    /// * `path` - The path of the INI file
    /// * `optional` - Indicates whether the INI file must exist
    pub fn new(path: PathBuf, optional: bool) -> Self {
        Self {
            path,
            optional,
            data: HashMap::with_capacity(0),
        }
    }
}

impl ConfigurationProvider for IniConfigurationProvider {
    fn get(&self, key: &str) -> Option<&str> {
        self.data.get(&key.to_uppercase()).map(|t| t.1.as_str())
    }

    fn load(&mut self) {
        if !self.path.is_file() {
            if self.optional {
                if !self.data.is_empty() {
                    self.data = HashMap::with_capacity(0);
                }

                return;
            } else {
                panic!(
                    "The configuration file '{}' was not found and is not optional.",
                    self.path.display()
                );
            }
        }

        let path = &self.path.display().to_string();
        let mut ini = Ini::new_cs();

        self.data = if let Ok(sections) = ini.load(&path) {
            let capacity = sections.iter().map(|p| p.1.len()).sum();
            let mut data = HashMap::with_capacity(capacity);

            for (section, pairs) in sections {
                for (key, value) in pairs {
                    let mut new_key = section.to_owned();
                    let new_value = value.unwrap_or_default();

                    new_key.push_str(ConfigurationPath::key_delimiter());
                    new_key.push_str(&key);
                    data.insert(new_key.to_uppercase(), (new_key, new_value));
                }
            }

            data
        } else {
            HashMap::with_capacity(0)
        }
    }

    fn child_keys(&self, earlier_keys: &mut Vec<String>, parent_path: Option<&str>) {
        accumulate_child_keys(&self.data, earlier_keys, parent_path)
    }
}

/// Represents a [configuration source](trait.ConfigurationSource.html) for INI files.
pub struct IniConfigurationSource {
    path: PathBuf,
    optional: bool,
}

impl IniConfigurationSource {
    /// Initializes a new INI file configuration source.
    ///
    /// # Arguments
    ///
    /// * `path` - The path of the INI file
    /// * `optional` - Indicates whether the INI file must exist
    pub fn new(path: &Path, optional: bool) -> Self {
        Self { path: path.to_path_buf(), optional }
    }
}

impl ConfigurationSource for IniConfigurationSource {
    fn build(&self, _builder: &dyn ConfigurationBuilder) -> Box<dyn ConfigurationProvider> {
        Box::new(IniConfigurationProvider::new(
            self.path.clone(),
            self.optional,
        ))
    }
}

pub mod ext {

    use super::*;

    /// Defines extension methods for the [ConfigurationBuilder](trait.ConfigurationBuilder.html) trait.
    pub trait IniConfigurationExtensions {
        /// Adds an INI file as a configuration source.
        ///
        /// # Arguments
        ///
        /// * `path` - The path of the INI file
        fn add_ini_file(&mut self, path: &Path) -> &mut Self;

        /// Adds an optional INI file as a configuration source.
        ///
        /// # Arguments
        ///
        /// * `path` - The path of the INI file
        fn add_optional_ini_file(&mut self, path: &Path) -> &mut Self;
    }

    impl IniConfigurationExtensions for dyn ConfigurationBuilder {
        fn add_ini_file(&mut self, path: &Path) -> &mut Self {
            self.add(Box::new(IniConfigurationSource::new(path, false)));
            self
        }

        fn add_optional_ini_file(&mut self, path: &Path) -> &mut Self {
            self.add(Box::new(IniConfigurationSource::new(path, true)));
            self
        }
    }

    impl<T: ConfigurationBuilder> IniConfigurationExtensions for T {
        fn add_ini_file(&mut self, path: &Path) -> &mut Self {
            self.add(Box::new(IniConfigurationSource::new(path, false)));
            self
        }

        fn add_optional_ini_file(&mut self, path: &Path) -> &mut Self {
            self.add(Box::new(IniConfigurationSource::new(path, true)));
            self
        }
    }
}
