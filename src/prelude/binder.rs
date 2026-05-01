use crate::{de, Configuration, Section};
use serde::de::DeserializeOwned;
use std::str::FromStr;

/// Represents [configuration](Configuration) binder for strongly-typed configurations.
pub trait Binder: Sized {
    /// Creates and returns a structure reified from the configuration.
    fn reify<T: DeserializeOwned>(&self) -> crate::Result<T>;

    /// Binds the configuration to the specified instance.
    ///
    /// # Arguments
    ///
    /// * `instance` - The instance to bind the configuration to
    fn bind<T: DeserializeOwned>(&self, instance: &mut T) -> crate::Result;

    /// Binds the specified configuration section to the provided instance.
    ///
    /// # Arguments
    ///
    /// * `key` - The key of the configuration section to bind
    /// * `instance` - The instance to bind the configuration to
    fn bind_at<T: DeserializeOwned>(&self, key: impl AsRef<str>, instance: &mut T) -> crate::Result;

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
binder!(Section<'_>);
