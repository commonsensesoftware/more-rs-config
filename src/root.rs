use crate::{Configuration, ConfigurationProvider, LoadError};
use std::fmt::{Debug, Formatter, Result as FormatResult};
use std::{borrow::Borrow, ops::Deref};

/// Defines the possible reload errors.
#[derive(PartialEq, Clone)]
pub enum ReloadError {
    /// Indicates one or more provider load errors occurred.
    Provider(Vec<(String, LoadError)>),

    /// Indicates reload cannot be performed because there
    /// are borrowed references. The number of references
    /// may be reported if known.
    Borrowed(Option<usize>),
}

impl Debug for ReloadError {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        match self {
            Self::Provider(errors) => {
                if errors.len() == 1 {
                    write!(f, "{} ({})", errors[0].1.message(), &errors[0].0)?;
                } else {
                    f.write_str("One or more load errors occurred:")?;

                    for (i, (provider, error)) in errors.iter().enumerate() {
                        write!(f, "\n  [{}]: {} ({})", (i+1), error.message(), provider)?;
                    }
                }
            }
            Self::Borrowed(count) => {
                write!(f, "Reload failed because the are")?;

                if let Some(value) = count {
                    write!(f, "{} ", value)?;
                }

                write!(f, " outstanding borrow references.")?;
            }
        }

        Ok(())
    }
}

/// Represents a configuration reload result.
pub type ReloadResult = std::result::Result<(), ReloadError>;

/// Represents the root of a [`Configuration`](crate::Configuration) hierarchy.
pub trait ConfigurationRoot:
    Configuration
    + AsRef<dyn Configuration>
    + Borrow<dyn Configuration>
    + Deref<Target = dyn Configuration>
    + Debug
{
    /// Force the configuration values to be reloaded from the underlying
    /// [`ConfigurationProvider`](crate::ConfigurationProvider) collection.
    fn reload(&mut self) -> ReloadResult;

    /// Gets the [`ConfigurationProvider`](crate::ConfigurationProvider) sequence for this configuration.
    fn providers(&self) -> Box<dyn ConfigurationProviderIterator + '_>;

    /// Converts the [`ConfigurationRoot``] into a [`Configuration``](crate::Configuration).
    fn as_config(&self) -> Box<dyn Configuration>;
}

/// Defines the behavior of an iterator over a
/// [`ConfigurationProvider`](crate::ConfigurationProvider) set.
pub trait ConfigurationProviderIterator<'a>:
    Iterator<Item = Box<dyn ConfigurationProvider + 'a>>
    + ExactSizeIterator<Item = Box<dyn ConfigurationProvider + 'a>>
    + DoubleEndedIterator<Item = Box<dyn ConfigurationProvider + 'a>>
{
}
