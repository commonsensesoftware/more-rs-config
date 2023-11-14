use crate::{Configuration, ConfigurationProvider, LoadError};
use std::fmt::{Debug, Formatter, Result as FormatResult};
use std::{borrow::Borrow, ops::Deref};

/// Defines the possible reload errors.
#[derive(PartialEq, Clone)]
pub enum ReloadError {
    /// Indicates one or more provider load errors occurred.
    Provider(Vec<(String, LoadError)>),

    /// Indicates reload cannot be performed because there
    /// are N outstanding borrow references.
    Borrowed(usize),
}

impl Debug for ReloadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        match self {
            Self::Provider(errors) => {
                if errors.len() == 1 {
                    f.write_str(errors[0].1.message())?;
                    f.write_str(" (")?;
                    f.write_str(&errors[0].0)?;
                    f.write_str(")")?;
                } else {
                    f.write_str("One or more load errors occurred:")?;

                    for (i, (provider, error)) in errors.iter().enumerate() {
                        f.write_str("\n")?;
                        f.write_str("  [")?;
                        (i + 1).fmt(f)?;
                        f.write_str("]: ")?;
                        f.write_str(error.message())?;
                        f.write_str(" (")?;
                        f.write_str(provider)?;
                        f.write_str(")")?;
                    }
                }
            }
            Self::Borrowed(count) => {
                f.write_str("Reload failed because the are ")?;
                count.fmt(f)?;
                f.write_str(" outstanding borrow references.")?;
            }
        }

        Ok(())
    }
}

/// Represents a configuration reload result.
pub type ReloadResult = std::result::Result<(), ReloadError>;

/// Represents the root of a [configuration](trait.Configuration.html) hierarchy.
pub trait ConfigurationRoot:
    Configuration
    + AsRef<dyn Configuration>
    + Borrow<dyn Configuration>
    + Deref<Target = dyn Configuration>
    + Debug
{
    /// Force the configuration values to be reloaded from the underlying [provider](trait.ConfigurationProvider.html).
    fn reload(&mut self) -> ReloadResult;

    /// Gets the [providers](trait.ConfigurationProvider.html) for this configuration.
    fn providers(&self) -> Box<dyn ConfigurationProviderIterator + '_>;

    /// Converts the [ConfigurationRoot](trait.ConfigurationRoot.html) into a [Configuration](trait.Configuration.html).
    fn as_config(&self) -> Box<dyn Configuration>;
}

/// Defines the behavior of an iterator over
/// [configuration providers](trait.ConfigurationProvider.html).
pub trait ConfigurationProviderIterator<'a>:
    Iterator<Item = &'a dyn ConfigurationProvider>
    + ExactSizeIterator<Item = &'a dyn ConfigurationProvider>
    + DoubleEndedIterator<Item = &'a dyn ConfigurationProvider>
{
}
