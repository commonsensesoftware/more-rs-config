#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod builder;
mod configuration;
mod error;
mod file;
mod merge;
mod properties;
mod provider;
mod root;
mod section;
mod settings;
mod source;

/// Contains chained configuration support.
#[cfg(feature = "chained")]
pub mod chained;

/// Contains command line configuration support.
#[cfg(feature = "cmd")]
pub mod cmd;

/// Contains strongly-typed configuration deserialization support.
#[cfg(feature = "binder")]
pub mod de;

/// Contains environment variable configuration support.
#[cfg(feature = "env")]
pub mod env;

/// Contains `*.ini` file configuration support.
#[cfg(feature = "ini")]
pub mod ini;

/// Contains `*.json` file configuration support.
#[cfg(feature = "json")]
pub mod json;

/// Contains in-memory configuration support.
#[cfg(feature = "mem")]
pub mod mem;

/// Provides configuration path utilities.
pub mod path;

/// Contains library prelude.
pub mod prelude;

/// Contains `*.xml` file configuration support.
#[cfg(feature = "xml")]
pub mod xml;

pub use builder::Builder;
pub use configuration::Configuration;
pub use error::Error;
pub use file::{FileSource, FileSourceBuilder};
pub use merge::Merge;
pub use properties::Properties;
pub use provider::Provider;
pub use root::Root;
pub use section::Section;
pub use settings::Settings;
pub use source::Source;

/// Represents the type alias for a configuration reference.
#[cfg(not(feature = "async"))]
pub type Ref<T> = std::rc::Rc<T>;

/// Represents the type alias for a configuration reference.
#[cfg(feature = "async")]
pub type Ref<T> = std::sync::Arc<T>;

/// Represents a configuration result.
pub type Result<T = ()> = std::result::Result<T, Error>;

/// Creates and returns a new [configuration builder](Builder)
#[inline]
pub fn builder() -> Builder {
    Builder::default()
}
