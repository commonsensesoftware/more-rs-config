use crate::{util::fmt_debug_view, *};
use std::any::Any;
use std::borrow::{Borrow, Cow};
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter, Result as FormatResult};
use std::iter::Map;
use std::ops::Deref;
use std::rc::Rc;
use std::slice::Iter;
use tokens::{ChangeToken, CompositeChangeToken, SharedChangeToken};

struct ProviderIter<'a, F>
where
    F: FnMut(&Box<dyn ConfigurationProvider>) -> &dyn ConfigurationProvider,
{
    items: &'a Vec<Box<dyn ConfigurationProvider>>,
    selector: F,
}

impl<'a, F> ProviderIter<'a, F>
where
    F: FnMut(&Box<dyn ConfigurationProvider>) -> &dyn ConfigurationProvider,
{
    pub fn new(items: &'a Vec<Box<dyn ConfigurationProvider>>, selector: F) -> Self {
        Self { items, selector }
    }
}

impl<'a, F> IntoIterator for ProviderIter<'a, F>
where
    F: FnMut(&Box<dyn ConfigurationProvider>) -> &dyn ConfigurationProvider,
{
    type Item = &'a dyn ConfigurationProvider;
    type IntoIter = Map<Iter<'a, Box<dyn ConfigurationProvider>>, F>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter().map(self.selector)
    }
}

impl<'a, F> ConfigurationProviderIterator<'a> for Map<Iter<'a, Box<dyn ConfigurationProvider>>, F> where
    F: FnMut(&Box<dyn ConfigurationProvider>) -> &dyn ConfigurationProvider
{
}

/// Represents the root of a configuration.
#[derive(Clone)]
pub struct DefaultConfigurationRoot {
    token: SharedChangeToken<CompositeChangeToken>,
    providers: Rc<Vec<Box<dyn ConfigurationProvider>>>,
}

impl DefaultConfigurationRoot {
    /// Initializes a new root configuration.
    ///
    /// # Arguments
    ///
    /// * `providers` - The list of [configuration providers](trait.ConfigurationProvider.html) used in the configuration
    pub fn new(mut providers: Vec<Box<dyn ConfigurationProvider>>) -> Self {
        let mut tokens = Vec::with_capacity(providers.len());

        for provider in providers.iter_mut() {
            provider.load();
            tokens.push(provider.reload_token());
        }

        Self {
            token: SharedChangeToken::new(CompositeChangeToken::new(tokens.into_iter())),
            providers: Rc::new(providers),
        }
    }
}

impl ConfigurationRoot for DefaultConfigurationRoot {
    fn reload(&mut self) -> bool {
        let reloaded;

        if let Some(providers) = Rc::get_mut(&mut self.providers) {
            let mut tokens = Vec::with_capacity(providers.len());

            for provider in providers {
                provider.load();
                tokens.push(provider.reload_token());
            }

            let new_token = SharedChangeToken::new(CompositeChangeToken::new(tokens.into_iter()));
            let old_token = std::mem::replace(&mut self.token, new_token);

            old_token.notify();
            reloaded = true
        } else {
            reloaded = false
        }

        reloaded
    }

    fn providers(&self) -> Box<dyn ConfigurationProviderIterator + '_> {
        Box::new(ProviderIter::new(&self.providers, |p| p.deref()).into_iter())
    }

    fn as_config(&self) -> Box<dyn Configuration> {
        Box::new(self.clone())
    }
}

impl Configuration for DefaultConfigurationRoot {
    fn get(&self, key: &str) -> Option<Cow<String>> {
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

    fn iter_relative(
        &self,
        make_paths_relative: bool,
    ) -> Box<dyn Iterator<Item = (String, String)>> {
        Box::new(ConfigurationIterator::new(self, make_paths_relative))
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
    /// * `root` - A reference to the [configuration root](trait.ConfigurationRoot.html)
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
    fn get(&self, key: &str) -> Option<Cow<String>> {
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

    fn iter_relative(
        &self,
        make_paths_relative: bool,
    ) -> Box<dyn Iterator<Item = (String, String)>> {
        Box::new(ConfigurationIterator::new(self, make_paths_relative))
    }
}

impl ConfigurationSection for DefaultConfigurationSection {
    fn key(&self) -> &str {
        ConfigurationPath::section_key(&self.path)
    }

    fn path(&self) -> &str {
        &self.path
    }

    fn value(&self) -> Cow<String> {
        self.root
            .get(&self.path)
            .unwrap_or(Cow::Owned(String::with_capacity(0)))
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

    fn build(&self) -> Box<dyn ConfigurationRoot> {
        Box::new(DefaultConfigurationRoot::new(
            self.sources.iter().map(|s| s.build(self)).collect(),
        ))
    }
}
