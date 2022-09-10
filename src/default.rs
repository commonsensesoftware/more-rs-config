use crate::{util::fmt_debug_view, *};
use std::any::Any;
use std::cell::{Cell, RefCell, UnsafeCell};
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Formatter, Result as FormatResult};
use std::rc::{Rc, Weak};
use std::sync::Mutex;
use tokens::{ChangeToken, SharedChangeToken};

struct Mediator {
    sync: Mutex<String>,
    token: RefCell<SharedChangeToken>,
    providers: UnsafeCell<Vec<Box<dyn ConfigurationProvider>>>,
    tokens: Cell<Vec<Box<dyn ChangeToken>>>,
    me: Weak<Mediator>,
}

impl Mediator {
    fn new(providers: Vec<Box<dyn ConfigurationProvider>>) -> Rc<Self> {
        Rc::new_cyclic(|me| {
            let tokens = Cell::new(Mediator::to_tokens(me, &providers));
            Self {
                sync: Mutex::default(),
                token: RefCell::default(),
                providers: UnsafeCell::new(providers),
                tokens,
                me: me.clone(),
            }
        })
    }

    fn to_tokens(
        me: &Weak<Mediator>,
        providers: &Vec<Box<dyn ConfigurationProvider>>,
    ) -> Vec<Box<dyn ChangeToken>> {
        providers
            .iter()
            .filter_map(|provider| provider.reload_token())
            .map(|token| {
                let this: Weak<Mediator> = me.clone();
                token.register(Box::new(move || this.upgrade().unwrap().raise_changed()));
                token
            })
            .collect()
    }

    fn providers(&self) -> &[Box<dyn ConfigurationProvider>] {
        unsafe { &*self.providers.get() }
    }

    fn reload(&self) {
        let _ = self.sync.lock().unwrap();

        // SAFETY: this is a 'chicken and egg' problem. there doesn't seem to be a better way that to moment.
        // 1. 'reload()' requires mutability, but mutability cannot be shared (e.g. Rc/Arc)
        // 2. interior mutability cannot be used because there is no way to then reference the slice of providers
        // 3. pushing interior mutability to providers breaks accessing values without copying the values
        //
        // reloading is not expected to be a common occurrence. while possible, it's also unlikely to be reloading
        // and reading a value at the same time. holding onto a configuration value reference instead of
        // copying/cloning it could be problematic. some providers (ex: in-memory) may do nothing when loaded or
        // do not support reloading. all providers are safely loaded at least once.
        //
        // consider refactoring to address this issue in the future.
        unsafe {
            for provider in &mut (*self.providers.get()).iter_mut() {
                provider.load();
            }
        }
    }

    fn reload_token(&self) -> Box<dyn ChangeToken> {
        let token;

        {
            let _ = self.sync.lock().unwrap();
            token = self.token.borrow().clone();
        }

        Box::new(token)
    }

    fn raise_changed(&self) {
        let tokens;

        unsafe {
            let _ = self.sync.lock().unwrap();
            tokens = Mediator::to_tokens(&self.me, &*self.providers.get());
        }

        let token = self.token.replace(SharedChangeToken::default());
        self.tokens.set(tokens);
        let callback = token.trigger().upgrade().unwrap();
        (callback)()
    }
}

/// Represents the root of a configuration.
#[derive(Clone)]
pub struct DefaultConfigurationRoot {
    mediator: Rc<Mediator>,
}

impl DefaultConfigurationRoot {
    /// Initializes a new root configuration.
    ///
    /// # Arguments
    ///
    /// * `providers` - The list of [configuration providers](trait.ConfigurationProvider.html) used in the configuration
    pub fn new(mut providers: Vec<Box<dyn ConfigurationProvider>>) -> Self {
        for provider in providers.iter_mut() {
            provider.load();
        }

        Self {
            mediator: Mediator::new(providers),
        }
    }
}

impl ConfigurationRoot for DefaultConfigurationRoot {
    fn reload(&mut self) {
        self.mediator.reload();
        self.mediator.raise_changed()
    }

    fn providers(&self) -> &[Box<dyn ConfigurationProvider>] {
        self.mediator.providers()
    }

    fn as_config(&self) -> &dyn Configuration {
        self
    }

    fn to_config(&self) -> Box<dyn Configuration> {
        Box::new(self.clone())
    }
}

impl Configuration for DefaultConfigurationRoot {
    fn get(&self, key: &str) -> Option<&str> {
        for provider in self.providers().iter().rev() {
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
        let keys: HashSet<_> = self
            .providers()
            .iter()
            .fold(Vec::new(), |mut earlier_keys, provider| {
                provider.child_keys(&mut earlier_keys, None);
                earlier_keys
            })
            .into_iter()
            .collect();

        keys.iter().map(|key| self.section(key)).collect()
    }

    fn reload_token(&self) -> Box<dyn ChangeToken> {
        self.mediator.reload_token()
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
    fn get(&self, key: &str) -> Option<&str> {
        self.root.get(&self.subkey(key))
    }

    fn section(&self, key: &str) -> Box<dyn ConfigurationSection> {
        self.root.section(&self.subkey(key))
    }

    fn children(&self) -> Vec<Box<dyn ConfigurationSection>> {
        let keys: HashSet<_> = self
            .root
            .providers()
            .iter()
            .fold(Vec::new(), |mut earlier_keys, provider| {
                provider.child_keys(&mut earlier_keys, Some(&self.path));
                earlier_keys
            })
            .into_iter()
            .collect();

        keys.iter().map(|key| self.section(key)).collect()
    }

    fn reload_token(&self) -> Box<dyn ChangeToken> {
        self.root.reload_token()
    }

    fn to_section(&self) -> Option<&dyn ConfigurationSection> {
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

    fn value(&self) -> &str {
        self.root.get(&self.path).unwrap_or("")
    }

    fn as_config(&self) -> &dyn Configuration {
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
