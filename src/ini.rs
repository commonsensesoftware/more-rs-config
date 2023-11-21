use crate::FileSource;
use crate::{
    util::accumulate_child_keys, ConfigurationBuilder, ConfigurationPath, ConfigurationProvider,
    ConfigurationSource, LoadError, LoadResult,
};
use configparser::ini::Ini;
use std::borrow::Cow;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tokens::{ChangeToken, FileChangeToken, SharedChangeToken, SingleChangeToken, Subscription};

struct InnerProvider {
    file: FileSource,
    data: RwLock<HashMap<String, (String, String)>>,
    token: RwLock<SharedChangeToken<SingleChangeToken>>,
}

impl InnerProvider {
    fn new(file: FileSource) -> Self {
        Self {
            file,
            data: RwLock::new(HashMap::with_capacity(0)),
            token: Default::default(),
        }
    }

    fn get(&self, key: &str) -> Option<Cow<String>> {
        self.data
            .read()
            .unwrap()
            .get(&key.to_uppercase())
            .map(|t| Cow::Owned(t.1.clone()))
    }

    fn reload_token(&self) -> Box<dyn ChangeToken> {
        Box::new(self.token.read().unwrap().clone())
    }

    fn load(&self, reload: bool) -> LoadResult {
        if !self.file.path.is_file() {
            if self.file.optional || reload {
                let mut data = self.data.write().unwrap();
                if !data.is_empty() {
                    *data = HashMap::with_capacity(0);
                }

                return Ok(());
            } else {
                return Err(LoadError::File {
                    message: format!(
                        "The configuration file '{}' was not found and is not optional.",
                        self.file.path.display()
                    ),
                    path: self.file.path.clone(),
                });
            }
        }

        let mut ini = Ini::new_cs();
        let data = if let Ok(sections) = ini.load(&self.file.path) {
            let capacity = sections.iter().map(|p| p.1.len()).sum();
            let mut map = HashMap::with_capacity(capacity);

            for (section, pairs) in sections {
                for (key, value) in pairs {
                    let mut new_key = section.to_owned();
                    let new_value = value.unwrap_or_default();

                    new_key.push_str(ConfigurationPath::key_delimiter());
                    new_key.push_str(&key);
                    map.insert(new_key.to_uppercase(), (new_key, new_value));
                }
            }

            map
        } else {
            HashMap::with_capacity(0)
        };

        *self.data.write().unwrap() = data;

        let previous = std::mem::replace(
            &mut *self.token.write().unwrap(),
            SharedChangeToken::default(),
        );

        previous.notify();
        Ok(())
    }

    fn child_keys(&self, earlier_keys: &mut Vec<String>, parent_path: Option<&str>) {
        let data = self.data.read().unwrap();
        accumulate_child_keys(&data, earlier_keys, parent_path)
    }
}

/// Represents a [configuration provider](trait.ConfigurationProvider.html) for INI files.
pub struct IniConfigurationProvider {
    inner: Arc<InnerProvider>,
    _subscription: Option<Box<dyn Subscription>>,
}

impl IniConfigurationProvider {
    /// Initializes a new INI file configuration provider.
    ///
    /// # Arguments
    ///
    /// * `file` - The [INI file](struct.FileSource.html) information
    pub fn new(file: FileSource) -> Self {
        let path = file.path.clone();
        let inner = Arc::new(InnerProvider::new(file));
        let subscription: Option<Box<dyn Subscription>> = if inner.file.reload_on_change {
            Some(Box::new(tokens::on_change(
                move || FileChangeToken::new(path.clone()),
                |state| {
                    let provider = state.unwrap();
                    std::thread::sleep(provider.file.reload_delay);
                    provider.load(true).ok();
                },
                Some(inner.clone()),
            )))
        } else {
            None
        };

        Self {
            inner,
            _subscription: subscription,
        }
    }
}

impl ConfigurationProvider for IniConfigurationProvider {
    fn get(&self, key: &str) -> Option<Cow<String>> {
        self.inner.get(key)
    }

    fn reload_token(&self) -> Box<dyn ChangeToken> {
        self.inner.reload_token()
    }

    fn load(&mut self) -> LoadResult {
        self.inner.load(false)
    }

    fn child_keys(&self, earlier_keys: &mut Vec<String>, parent_path: Option<&str>) {
        self.inner.child_keys(earlier_keys, parent_path)
    }
}

/// Represents a [configuration source](trait.ConfigurationSource.html) for INI files.
pub struct IniConfigurationSource {
    file: FileSource,
}

impl IniConfigurationSource {
    /// Initializes a new INI file configuration source.
    ///
    /// # Arguments
    ///
    /// * `file` - The [INI file](struct.FileSource.html) information
    pub fn new(file: FileSource) -> Self {
        Self { file }
    }
}

impl ConfigurationSource for IniConfigurationSource {
    fn build(&self, _builder: &dyn ConfigurationBuilder) -> Box<dyn ConfigurationProvider> {
        Box::new(IniConfigurationProvider::new(self.file.clone()))
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
        /// * `file` - The [INI file](struct.FileSource.html) information
        fn add_ini_file<T: Into<FileSource>>(&mut self, file: T) -> &mut Self;
    }

    impl IniConfigurationExtensions for dyn ConfigurationBuilder {
        fn add_ini_file<T: Into<FileSource>>(&mut self, file: T) -> &mut Self {
            self.add(Box::new(IniConfigurationSource::new(file.into())));
            self
        }
    }

    impl<T: ConfigurationBuilder> IniConfigurationExtensions for T {
        fn add_ini_file<F: Into<FileSource>>(&mut self, file: F) -> &mut Self {
            self.add(Box::new(IniConfigurationSource::new(file.into())));
            self
        }
    }
}
