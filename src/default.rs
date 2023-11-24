use crate::{util::fmt_debug_view, *};
use cfg_if::cfg_if;
use std::any::Any;
use std::borrow::Borrow;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter, Result as FormatResult};
use std::ops::Deref;
use tokens::{ChangeToken, CompositeChangeToken, SharedChangeToken};

cfg_if! {
    if #[cfg(feature = "async")] {
        use std::sync::Arc;

        type Pc<T> = std::sync::Arc<T>;
        type Mut<T> = std::sync::RwLock<T>;
        type Ref<'a, T> = Arc<std::sync::RwLockReadGuard<'a, T>>;
    } else {
        use std::cell::Ref;

        type Pc<T> = std::rc::Rc<T>;
        type Mut<T> = std::cell::RefCell<T>;
    }
}

struct ProviderItem<'a> {
    index: usize,
    name: String,
    items: Ref<'a, Vec<Box<dyn ConfigurationProvider + 'a>>>,
}

impl<'a> ProviderItem<'a> {
    fn new(
        items: Ref<'a, Vec<Box<dyn ConfigurationProvider + 'a>>>,
        index: usize,
        name: String,
    ) -> Self {
        Self { index, name, items }
    }
}

impl ConfigurationProvider for ProviderItem<'_> {
    fn get(&self, key: &str) -> Option<Value> {
        self.items[self.index].get(key)
    }

    fn child_keys(&self, earlier_keys: &mut Vec<String>, parent_path: Option<&str>) {
        self.items[self.index].child_keys(earlier_keys, parent_path)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn reload_token(&self) -> Box<dyn ChangeToken> {
        self.items[self.index].reload_token()
    }
}

struct ProviderIter<'a> {
    head: usize,
    tail: usize,
    items: Ref<'a, Vec<Box<dyn ConfigurationProvider>>>,
}

impl<'a> ProviderIter<'a> {
    fn new(items: Ref<'a, Vec<Box<dyn ConfigurationProvider>>>) -> Self {
        Self {
            head: 0,
            tail: items.len(),
            items,
        }
    }
}

impl<'a> Iterator for ProviderIter<'a> {
    type Item = Box<dyn ConfigurationProvider + 'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.head < self.items.len() {
            let i = self.head;
            let name = self.items[i].name().to_owned();
            self.head += 1;

            cfg_if! {
                if #[cfg(feature = "async")] {
                    Some(Box::new(ProviderItem::new(
                        self.items.clone(),
                        i,
                        name,
                    )))
                } else {
                    Some(Box::new(ProviderItem::new(
                        Ref::clone(&self.items),
                        i,
                        name,
                    )))
                }
            }
        } else {
            None
        }
    }
}

impl ExactSizeIterator for ProviderIter<'_> {
    fn len(&self) -> usize {
        self.items.len()
    }
}

impl DoubleEndedIterator for ProviderIter<'_> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.tail > 0 {
            self.tail -= 1;
            let name = self.items[self.tail].name().to_owned();

            cfg_if! {
                if #[cfg(feature = "async")] {
                    Some(Box::new(ProviderItem::new(
                        self.items.clone(),
                        self.tail,
                        name,
                    )))
                } else {
                    Some(Box::new(ProviderItem::new(
                        Ref::clone(&self.items),
                        self.tail,
                        name,
                    )))
                }
            }
        } else {
            None
        }
    }
}

impl<'a> ConfigurationProviderIterator<'a> for ProviderIter<'a> {}

/// Represents the root of a configuration.
#[derive(Clone)]
pub struct DefaultConfigurationRoot {
    token: SharedChangeToken<CompositeChangeToken>,
    providers: Pc<Mut<Vec<Box<dyn ConfigurationProvider>>>>,
}

impl DefaultConfigurationRoot {
    /// Initializes a new root configuration.
    ///
    /// # Arguments
    ///
    /// * `providers` - The [`ConfigurationProvider`](crate::ConfigurationProvider) list used in the configuration
    pub fn new(mut providers: Vec<Box<dyn ConfigurationProvider>>) -> Result<Self, ReloadError> {
        let mut errors = Vec::new();
        let mut tokens = Vec::with_capacity(providers.len());

        for provider in providers.iter_mut() {
            let result = provider.load();

            if let Err(error) = result {
                errors.push((provider.name().to_owned(), error));
            }

            tokens.push(provider.reload_token());
        }

        if errors.is_empty() {
            Ok(Self {
                token: SharedChangeToken::new(CompositeChangeToken::new(tokens.into_iter())),
                providers: Pc::new(providers.into()),
            })
        } else {
            Err(ReloadError::Provider(errors))
        }
    }
}

impl ConfigurationRoot for DefaultConfigurationRoot {
    fn reload(&mut self) -> ReloadResult {
        let borrowed = (Pc::strong_count(&self.providers) - 1) + Pc::weak_count(&self.providers);

        cfg_if! {
            if #[cfg(feature = "async")] {
                let result = self.providers.try_write();
            } else {
                let result = self.providers.try_borrow_mut();
            }
        }

        if let Ok(mut providers) = result {
            let mut errors = Vec::new();
            let mut tokens = Vec::with_capacity(providers.len());

            for provider in providers.iter_mut() {
                let result = provider.load();

                if let Err(error) = result {
                    errors.push((provider.name().to_owned(), error));
                }

                tokens.push(provider.reload_token());
            }

            let new_token = SharedChangeToken::new(CompositeChangeToken::new(tokens.into_iter()));
            let old_token = std::mem::replace(&mut self.token, new_token);

            old_token.notify();

            if errors.is_empty() {
                Ok(())
            } else {
                Err(ReloadError::Provider(errors))
            }
        } else {
            Err(ReloadError::Borrowed(Some(borrowed)))
        }
    }

    fn providers(&self) -> Box<dyn ConfigurationProviderIterator + '_> {
        cfg_if! {
            if #[cfg(feature = "async")] {
                Box::new(ProviderIter::new(self.providers.deref().read().unwrap().into()))
            } else {
                Box::new(ProviderIter::new(self.providers.deref().borrow()))
            }
        }
    }

    fn as_config(&self) -> Box<dyn Configuration> {
        Box::new(self.clone())
    }
}

impl Configuration for DefaultConfigurationRoot {
    fn get(&self, key: &str) -> Option<Value> {
        for provider in self.providers().rev() {
            if let Some(value) = provider.get(key) {
                return Some(value);
            }
        }

        None
    }

    fn section(&self, key: &str) -> Box<dyn ConfigurationSection> {
        Box::new(DefaultConfigurationSection::new(
            Box::new(self.clone()),
            key,
        ))
    }

    fn children(&self) -> Vec<Box<dyn ConfigurationSection>> {
        self.providers()
            .fold(Vec::new(), |mut earlier_keys, provider| {
                provider.child_keys(&mut earlier_keys, None);
                earlier_keys
            })
            .into_iter()
            .collect::<HashSet<_>>()
            .iter()
            .map(|key| self.section(key))
            .collect()
    }

    fn reload_token(&self) -> Box<dyn ChangeToken> {
        Box::new(self.token.clone())
    }

    fn iter(&self, path: Option<ConfigurationPath>) -> Box<dyn Iterator<Item = (String, Value)>> {
        Box::new(ConfigurationIterator::new(
            self,
            path.unwrap_or(ConfigurationPath::Absolute),
        ))
    }
}

impl Debug for DefaultConfigurationRoot {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
        fmt_debug_view(self, formatter)
    }
}

impl<'a> AsRef<dyn Configuration + 'a> for DefaultConfigurationRoot {
    fn as_ref(&self) -> &(dyn Configuration + 'a) {
        self
    }
}

impl<'a> Borrow<dyn Configuration + 'a> for DefaultConfigurationRoot {
    fn borrow(&self) -> &(dyn Configuration + 'a) {
        self
    }
}

impl Deref for DefaultConfigurationRoot {
    type Target = dyn Configuration;

    fn deref(&self) -> &Self::Target {
        self
    }
}

cfg_if! {
    if #[cfg(feature = "async")] {
        unsafe impl Send for DefaultConfigurationRoot {}
        unsafe impl Sync for DefaultConfigurationRoot {}
    }
}

/// Represent a configuration section.
pub struct DefaultConfigurationSection {
    root: Box<dyn ConfigurationRoot>,
    path: String,
}

impl DefaultConfigurationSection {
    /// Initializes a new configuration section.
    ///
    /// # Arguments
    ///
    /// * `root` - A reference to the [`ConfigurationRoot`](crate::ConfigurationRoot)
    /// * `path` - The path of the configuration section
    pub fn new(root: Box<dyn ConfigurationRoot>, path: &str) -> Self {
        Self {
            root,
            path: path.to_owned(),
        }
    }

    #[inline]
    fn subkey(&self, key: &str) -> String {
        ConfigurationPath::combine(&[&self.path, key])
    }
}

impl Configuration for DefaultConfigurationSection {
    fn get(&self, key: &str) -> Option<Value> {
        self.root.get(&self.subkey(key))
    }

    fn section(&self, key: &str) -> Box<dyn ConfigurationSection> {
        self.root.section(&self.subkey(key))
    }

    fn children(&self) -> Vec<Box<dyn ConfigurationSection>> {
        self.root
            .providers()
            .fold(Vec::new(), |mut earlier_keys, provider| {
                provider.child_keys(&mut earlier_keys, Some(&self.path));
                earlier_keys
            })
            .into_iter()
            .collect::<HashSet<_>>()
            .iter()
            .map(|key| self.section(key))
            .collect()
    }

    fn reload_token(&self) -> Box<dyn ChangeToken> {
        self.root.reload_token()
    }

    fn as_section(&self) -> Option<&dyn ConfigurationSection> {
        Some(self)
    }

    fn iter(&self, path: Option<ConfigurationPath>) -> Box<dyn Iterator<Item = (String, Value)>> {
        Box::new(ConfigurationIterator::new(
            self,
            path.unwrap_or(ConfigurationPath::Absolute),
        ))
    }
}

impl ConfigurationSection for DefaultConfigurationSection {
    fn key(&self) -> &str {
        ConfigurationPath::section_key(&self.path)
    }

    fn path(&self) -> &str {
        &self.path
    }

    fn value(&self) -> Value {
        self.root.get(&self.path).unwrap_or_default()
    }
}

impl<'a> AsRef<dyn Configuration + 'a> for DefaultConfigurationSection {
    fn as_ref(&self) -> &(dyn Configuration + 'a) {
        self
    }
}

impl<'a> Borrow<dyn Configuration + 'a> for DefaultConfigurationSection {
    fn borrow(&self) -> &(dyn Configuration + 'a) {
        self
    }
}

impl Deref for DefaultConfigurationSection {
    type Target = dyn Configuration;

    fn deref(&self) -> &Self::Target {
        self
    }
}

/// Represents a configuration builder.
#[derive(Default)]
pub struct DefaultConfigurationBuilder {
    /// Gets the associated configuration sources.
    pub sources: Vec<Box<dyn ConfigurationSource>>,

    /// Gets the properties that can be passed to configuration sources.
    pub properties: HashMap<String, Box<dyn Any>>,
}

impl DefaultConfigurationBuilder {
    /// Initializes a new, default configuration builder.
    pub fn new() -> Self {
        Self::default()
    }
}

impl ConfigurationBuilder for DefaultConfigurationBuilder {
    fn properties(&self) -> &HashMap<String, Box<dyn Any>> {
        &self.properties
    }

    fn sources(&self) -> &[Box<dyn ConfigurationSource>] {
        &self.sources
    }

    fn add(&mut self, source: Box<dyn ConfigurationSource>) {
        self.sources.push(source)
    }

    fn build(&self) -> Result<Box<dyn ConfigurationRoot>, ReloadError> {
        Ok(Box::new(DefaultConfigurationRoot::new(
            self.sources.iter().map(|s| s.build(self)).collect(),
        )?))
    }
}
