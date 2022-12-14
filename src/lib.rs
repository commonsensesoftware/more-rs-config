#![doc = include_str!("README.md")]

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

#[cfg(feature = "binder")]
mod binder;

#[cfg(feature = "binder")]
mod de;

pub use builder::*;
pub use configuration::*;
pub use path::*;
pub use provider::*;
pub use root::*;
pub use section::ConfigurationSection;
pub use source::*;

#[cfg(feature = "util")]
pub use util::*;

#[cfg(feature = "chained")]
pub use chained::{ChainedConfigurationProvider, ChainedConfigurationSource};

#[cfg(feature = "std")]
pub use default::*;

#[cfg(feature = "mem")]
pub use memory::{MemoryConfigurationProvider, MemoryConfigurationSource};

#[cfg(feature = "env")]
pub use env::{EnvironmentVariablesConfigurationProvider, EnvironmentVariablesConfigurationSource};

#[cfg(feature = "ini")]
pub use ini::{IniConfigurationProvider, IniConfigurationSource};

#[cfg(feature = "json")]
pub use json::{JsonConfigurationProvider, JsonConfigurationSource};

#[cfg(feature = "cmd")]
pub use cmd::{CommandLineConfigurationProvider, CommandLineConfigurationSource};

/// Contains configuration extension methods.
pub mod ext {

    use super::*;

    #[cfg(feature = "chained")]
    pub use chained::ext::*;

    #[cfg(feature = "env")]
    pub use env::ext::*;

    #[cfg(feature = "ini")]
    pub use ini::ext::*;

    #[cfg(feature = "json")]
    pub use json::ext::*;

    #[cfg(feature = "mem")]
    pub use memory::ext::*;

    #[cfg(feature = "cmd")]
    pub use cmd::ext::*;

    #[cfg(feature = "binder")]
    pub use binder::*;

    #[cfg(feature = "binder")]
    pub use de::*;

    pub use section::ext::*;
}
