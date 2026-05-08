use crate::{path, prelude::Binder, settings, Builder, Reloadable, Section, Settings};
use arc_swap::ArcSwap;
use serde::de::DeserializeOwned;
use std::collections::VecDeque;
use std::fmt::{self, Debug, Display, Formatter, Write};
use std::str::FromStr;
use std::{any::Any, sync::Arc};
use tokens::{ChangeToken, CompositeChangeToken, Registration, SharedChangeToken, SingleChangeToken};
use tracing::{error, trace};

/// Represents a configuration.
#[derive(Clone)]
pub struct Configuration {
    pub(crate) settings: Settings,
    token: SharedChangeToken<CompositeChangeToken>,
    pub(crate) providers: Vec<String>,
}

impl Configuration {
    /// Initializes a new [Configuration].
    ///
    /// # Arguments
    ///
    /// * `settings` - The configuration [settings](Settings)
    /// * `tokens` - The [sequence](Iterator) of [change tokens](ChangeToken) associated with the configuration
    /// * `providers` - The names of the providers that generated the configuration
    #[inline]
    pub fn new(
        settings: Settings,
        tokens: impl IntoIterator<Item = Box<dyn ChangeToken>>,
        providers: Vec<String>,
    ) -> Self {
        Self {
            settings,
            token: SharedChangeToken::new(CompositeChangeToken::new(tokens.into_iter())),
            providers,
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
            let Some(key) = path::next(path, None) else {
                continue;
            };

            if !keys.iter().any(|k: &String| k.eq_ignore_ascii_case(key)) {
                keys.push(key.to_owned());
            }
        }

        keys.into_iter().map(|key| self.section(key)).collect()
    }

    /// Returns a [change token](ChangeToken) that indicates when the configuration has changed.
    #[inline]
    pub fn change_token(&self) -> impl ChangeToken {
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

impl Debug for Configuration {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.providers.is_empty() {
            Debug::fmt(&self.settings, f)
        } else {
            let mut sections: VecDeque<_> = self.sections().into_iter().map(|s| (0, s)).collect();

            while let Some((depth, section)) = sections.pop_front() {
                f.write_str(&"  ".repeat(depth))?;
                Display::fmt(&section, f)?;

                for (i, child) in section.sections().into_iter().map(|s| (depth + 1, s)).enumerate() {
                    sections.insert(i, child);
                }

                if !sections.is_empty() {
                    f.write_char('\n')?;
                }
            }

            Ok(())
        }
    }
}

impl Display for Configuration {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.settings, f)
    }
}

fn on_changed(state: Option<Arc<dyn Any + Send + Sync + 'static>>) {
    let Some(state) = state else {
        return;
    };

    let inner = state.downcast_ref::<Inner>().expect("received state other than Inner");

    match inner.builder.build() {
        Ok(config) => {
            let registration = inner
                .config
                .load()
                .change_token()
                .register(Box::new(on_changed), Some(state.clone()));
            let token = inner.token.swap(Arc::new(SharedChangeToken::default()));

            inner.config.store(Arc::new(config));
            inner.registration.store(Arc::new(registration));

            trace!("Reloaded the configuration");

            token.notify();
        }
        Err(error) => error!("Failed to reload the configuration. {error:?}"),
    }
}

struct Inner {
    builder: Builder,
    config: ArcSwap<Configuration>,
    registration: ArcSwap<Registration>,
    token: ArcSwap<SharedChangeToken<SingleChangeToken>>,
}

/// Represents a reloadable [configuration](Configuration).
pub struct ReloadableConfiguration(Arc<Inner>);

impl ReloadableConfiguration {
    /// Initializes a new [ReloadableConfiguration].
    ///
    /// # Arguments
    ///
    /// * `builder` - The [builder](Builder) used to reload the configuration
    /// * `config` - The initial [configuration](Configuration)
    pub fn new(builder: Builder, configuration: Configuration) -> Self {
        let inner = Arc::new(Inner {
            builder,
            config: ArcSwap::from_pointee(configuration),
            registration: ArcSwap::from_pointee(Registration::none()),
            token: ArcSwap::from_pointee(SharedChangeToken::default()),
        });
        let registration = inner
            .config
            .load()
            .change_token()
            .register(Box::new(on_changed), Some(inner.clone()));

        inner.registration.store(registration.into());
        Self(inner)
    }

    /// Gets the current [configuration](Configuration).
    ///
    /// # Remarks
    ///
    /// This method will reload the [configuration](Configuration) if it has changed. If the reload operation fails,
    /// then the error is logged and the previous [configuration](Configuration) is retained.
    #[inline]
    pub fn current(&self) -> Arc<Configuration> {
        self.0.config.load_full()
    }

    /// Creates and returns a structure reified from the configuration.
    #[inline]
    pub fn reify<T: DeserializeOwned>(&self) -> crate::Result<T> {
        self.0.config.load().reify()
    }

    /// Binds the configuration to the specified instance.
    ///
    /// # Arguments
    ///
    /// * `instance` - The instance to bind the configuration to
    #[inline]
    pub fn bind<T: DeserializeOwned>(&self, instance: &mut T) -> crate::Result {
        self.0.config.load().bind(instance)
    }

    /// Binds the specified configuration section to the provided instance.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the configuration section to bind
    /// * `instance` - The instance to bind the configuration to
    #[inline]
    pub fn bind_at<T: DeserializeOwned>(&self, key: impl AsRef<str>, instance: &mut T) -> crate::Result {
        self.0.config.load().bind_at(key, instance)
    }

    /// Gets a typed value from the configuration.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the value to retrieve
    #[inline]
    pub fn get_value<T: FromStr>(&self, key: impl AsRef<str>) -> Result<Option<T>, T::Err> {
        self.0.config.load().get_value(key)
    }

    /// Gets an optional, typed value from the configuration.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the value to retrieve
    #[inline]
    pub fn get_value_or_default<T: FromStr + Default>(&self, key: impl AsRef<str>) -> Result<T, T::Err> {
        self.0.config.load().get_value_or_default(key)
    }
}

impl Clone for ReloadableConfiguration {
    #[inline]
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl Reloadable for ReloadableConfiguration {
    #[inline]
    fn can_reload(&self) -> bool {
        true
    }

    #[inline]
    fn reload_token(&self) -> impl ChangeToken + 'static {
        (**self.0.token.load()).clone()
    }
}

impl Debug for ReloadableConfiguration {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(&*self.current(), f)
    }
}

impl Display for ReloadableConfiguration {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&*self.current(), f)
    }
}

impl From<ReloadableConfiguration> for Arc<Configuration> {
    #[inline]
    fn from(rc: ReloadableConfiguration) -> Self {
        rc.current()
    }
}

impl From<&ReloadableConfiguration> for Arc<Configuration> {
    #[inline]
    fn from(rc: &ReloadableConfiguration) -> Self {
        rc.current()
    }
}

impl TryFrom<Builder> for ReloadableConfiguration {
    type Error = crate::Error;

    fn try_from(builder: Builder) -> crate::Result<Self> {
        let config = builder.build()?;
        Ok(Self::new(builder, config))
    }
}
