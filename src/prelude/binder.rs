use crate::{de, section::OwnedSection, Configuration, ReloadableConfiguration, Section};
use serde::de::DeserializeOwned;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;

/// Represents [configuration](Configuration) binder for strongly-typed configurations.
pub trait Binder: Sized {
    /// Creates and returns a structure reified from the configuration.
    fn reify<T: DeserializeOwned>(&self) -> crate::Result<T>;

    /// Creates and returns a structure reified from the configuration.
    ///
    /// # Remarks
    ///
    /// This function panics the reify operation fails.
    fn reify_unchecked<T: DeserializeOwned>(&self) {
        if let Err(error) = self.reify::<T>() {
            panic!("{}", error);
        }
    }

    /// Binds the configuration to the specified instance.
    ///
    /// # Arguments
    ///
    /// * `instance` - The instance to bind the configuration to
    fn bind<T: DeserializeOwned>(&self, instance: &mut T) -> crate::Result;

    /// Binds the configuration to the specified instance.
    ///
    /// # Arguments
    ///
    /// * `instance` - The instance to bind the configuration to
    ///
    /// # Remarks
    ///
    /// This function panics the bind operation fails.
    fn bind_unchecked<T: DeserializeOwned>(&self, instance: &mut T) {
        if let Err(error) = self.bind(instance) {
            panic!("{}", error);
        }
    }

    /// Binds the specified configuration section to the provided instance.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the configuration section to bind
    /// * `instance` - The instance to bind the configuration to
    fn bind_at<T: DeserializeOwned>(&self, key: impl AsRef<str>, instance: &mut T) -> crate::Result;

    /// Binds the specified configuration section to the provided instance.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the configuration section to bind
    /// * `instance` - The instance to bind the configuration to
    ///
    /// # Remarks
    ///
    /// This function panics the bind operation fails.
    fn bind_at_unchecked<T: DeserializeOwned>(&self, key: impl AsRef<str>, instance: &mut T) {
        if let Err(error) = self.bind_at(key, instance) {
            panic!("{}", error);
        }
    }

    /// Gets a typed value from the configuration.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the value to retrieve
    fn get_value<T: FromStr>(&self, key: impl AsRef<str>) -> Result<Option<T>, T::Err>;

    /// Gets an optional, typed value from the configuration.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the value to retrieve
    fn get_value_or_default<T: FromStr + Default>(&self, key: impl AsRef<str>) -> Result<T, T::Err>;
}

macro_rules! binder {
    ($type:ty) => {
        impl Binder for $type {
            #[inline]
            fn reify<T: DeserializeOwned>(&self) -> crate::Result<T> {
                Ok(de::from::<T>(self.sections())?)
            }

            fn bind<T: DeserializeOwned>(&self, instance: &mut T) -> crate::Result {
                Ok(de::bind(self.sections(), instance)?)
            }

            fn bind_at<T: DeserializeOwned>(&self, key: impl AsRef<str>, instance: &mut T) -> crate::Result {
                let section = self.section(key.as_ref());

                if section.exists() {
                    Ok(de::bind(section.sections(), instance)?)
                } else {
                    Ok(())
                }
            }

            fn get_value<T: FromStr>(&self, key: impl AsRef<str>) -> Result<Option<T>, T::Err> {
                let section = self.section(key.as_ref());
                let value = if section.exists() {
                    Some(T::from_str(section.value())?)
                } else {
                    None
                };

                Ok(value)
            }

            fn get_value_or_default<T: FromStr + Default>(&self, key: impl AsRef<str>) -> Result<T, T::Err> {
                let section = self.section(key.as_ref());
                let value = if section.exists() {
                    T::from_str(section.value())?
                } else {
                    T::default()
                };

                Ok(value)
            }
        }
    };
}

binder!(Configuration);
binder!(Arc<Configuration>);
binder!(Rc<Configuration>);
binder!(Section<'_>);
binder!(OwnedSection);

impl Binder for ReloadableConfiguration {
    #[inline]
    fn reify<T: DeserializeOwned>(&self) -> crate::Result<T> {
        self.reify()
    }

    #[inline]
    fn bind<T: DeserializeOwned>(&self, instance: &mut T) -> crate::Result {
        self.bind(instance)
    }

    #[inline]
    fn bind_at<T: DeserializeOwned>(&self, key: impl AsRef<str>, instance: &mut T) -> crate::Result {
        self.bind_at(key, instance)
    }

    #[inline]
    fn get_value<T: FromStr>(&self, key: impl AsRef<str>) -> Result<Option<T>, T::Err> {
        self.get_value(key)
    }

    #[inline]
    fn get_value_or_default<T: FromStr + Default>(&self, key: impl AsRef<str>) -> Result<T, T::Err> {
        self.get_value_or_default(key)
    }
}
