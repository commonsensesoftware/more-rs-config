#![doc = include_str!("README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

/// Represents the type alias for a configuration value.
#[cfg(not(feature = "async"))]
pub type Value = std::rc::Rc<String>;

/// Represents the type alias for a configuration value.
#[cfg(feature = "async")]
pub type Value = std::sync::Arc<String>;

mod builder;
mod configuration;
mod path;
mod provider;
mod root;
mod section;
mod source;

/// Contains configuration utility functions.
#[cfg(feature = "util")]
pub mod util;

#[cfg(feature = "chained")]
mod chained;

#[cfg(feature = "std")]
mod default;

#[cfg(feature = "mem")]
mod memory;

#[cfg(feature = "env")]
mod env;

#[cfg(feature = "ini")]
mod ini;

#[cfg(feature = "json")]
mod json;

#[cfg(feature = "cmd")]
mod cmd;

#[cfg(feature = "xml")]
mod xml;

#[cfg(feature = "binder")]
mod binder;

#[cfg(feature = "binder")]
mod de;

#[cfg(feature = "struct")]
mod r#struct;

mod file;
pub use builder::*;
pub use configuration::*;
pub use file::*;
pub use path::*;
pub use provider::*;
pub use root::*;
pub use section::ConfigurationSection;
pub use source::*;

#[cfg(feature = "util")]
#[cfg_attr(docsrs, doc(cfg(feature = "util")))]
pub use util::*;

#[cfg(feature = "chained")]
#[cfg_attr(docsrs, doc(cfg(feature = "chained")))]
pub use chained::{ChainedConfigurationProvider, ChainedConfigurationSource};

#[cfg(feature = "std")]
#[cfg_attr(docsrs, doc(cfg(feature = "std")))]
pub use default::*;

#[cfg(feature = "mem")]
#[cfg_attr(docsrs, doc(cfg(feature = "mem")))]
pub use memory::{MemoryConfigurationProvider, MemoryConfigurationSource};

#[cfg(feature = "env")]
#[cfg_attr(docsrs, doc(cfg(feature = "env")))]
pub use env::{EnvironmentVariablesConfigurationProvider, EnvironmentVariablesConfigurationSource};

#[cfg(feature = "ini")]
#[cfg_attr(docsrs, doc(cfg(feature = "ini")))]
pub use ini::{IniConfigurationProvider, IniConfigurationSource};

#[cfg(feature = "json")]
#[cfg_attr(docsrs, doc(cfg(feature = "json")))]
pub use json::{JsonConfigurationProvider, JsonConfigurationSource};

#[cfg(feature = "cmd")]
#[cfg_attr(docsrs, doc(cfg(feature = "cmd")))]
pub use cmd::{CommandLineConfigurationProvider, CommandLineConfigurationSource};

#[cfg(feature = "xml")]
#[cfg_attr(docsrs, doc(cfg(feature = "xml")))]
pub use xml::{XmlConfigurationProvider, XmlConfigurationSource};

#[cfg(feature = "struct")]
#[cfg_attr(docsrs, doc(cfg(feature = "struct")))]
pub use r#struct::{StructConfigurationProvider, StructConfigurationSource};

/// Contains configuration extension methods.
pub mod ext {

    use super::*;

    #[cfg(feature = "chained")]
    #[cfg_attr(docsrs, doc(cfg(feature = "chained")))]
    pub use chained::ext::*;

    #[cfg(feature = "env")]
    #[cfg_attr(docsrs, doc(cfg(feature = "env")))]
    pub use env::ext::*;

    #[cfg(feature = "ini")]
    #[cfg_attr(docsrs, doc(cfg(feature = "ini")))]
    pub use ini::ext::*;

    #[cfg(feature = "json")]
    #[cfg_attr(docsrs, doc(cfg(feature = "json")))]
    pub use json::ext::*;

    #[cfg(feature = "mem")]
    #[cfg_attr(docsrs, doc(cfg(feature = "mem")))]
    pub use memory::ext::*;

    #[cfg(feature = "cmd")]
    #[cfg_attr(docsrs, doc(cfg(feature = "cmd")))]
    pub use cmd::ext::*;

    #[cfg(feature = "xml")]
    #[cfg_attr(docsrs, doc(cfg(feature = "xml")))]
    pub use super::xml::ext::*;

    #[cfg(feature = "binder")]
    #[cfg_attr(docsrs, doc(cfg(feature = "binder")))]
    pub use binder::*;

    #[cfg(feature = "binder")]
    #[cfg_attr(docsrs, doc(cfg(feature = "binder")))]
    pub use de::*;

    #[cfg(feature = "struct")]
    #[cfg_attr(docsrs, doc(cfg(feature = "struct")))]
    pub use r#struct::ext::*;

    pub use section::ext::*;
    pub use file::ext::*;
}
