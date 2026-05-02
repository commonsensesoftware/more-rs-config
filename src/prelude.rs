#[cfg(feature = "binder")]
mod binder;

#[cfg(feature = "chained")]
mod chained;

#[cfg(feature = "cmd")]
mod cmd;

#[cfg(feature = "env")]
mod env;

mod file;

#[cfg(feature = "ini")]
mod ini;

#[cfg(feature = "json")]
mod json;

#[cfg(feature = "mem")]
mod mem;

#[cfg(feature = "typed")]
mod typed;

#[cfg(feature = "xml")]
mod xml;

#[cfg(feature = "yaml")]
mod yaml;

#[cfg(feature = "binder")]
#[cfg_attr(docsrs, doc(cfg(feature = "binder")))]
pub use binder::Binder;

#[cfg(feature = "chained")]
#[cfg_attr(docsrs, doc(cfg(feature = "chained")))]
pub use chained::ChainedExt;

#[cfg(feature = "cmd")]
#[cfg_attr(docsrs, doc(cfg(feature = "cmd")))]
pub use cmd::CommandLineExt;

#[cfg(feature = "env")]
#[cfg_attr(docsrs, doc(cfg(feature = "env")))]
pub use env::EnvVarsExt;

pub use file::FileSourceBuilderExt;

#[cfg(feature = "ini")]
#[cfg_attr(docsrs, doc(cfg(feature = "ini")))]
pub use ini::IniExt;

#[cfg(feature = "json")]
#[cfg_attr(docsrs, doc(cfg(feature = "json")))]
pub use json::JsonExt;

#[cfg(feature = "mem")]
#[cfg_attr(docsrs, doc(cfg(feature = "mem")))]
pub use mem::MemoryExt;

#[cfg(feature = "typed")]
#[cfg_attr(docsrs, doc(cfg(feature = "typed")))]
pub use typed::TypedExt;

#[cfg(feature = "xml")]
#[cfg_attr(docsrs, doc(cfg(feature = "xml")))]
pub use xml::XmlExt;

#[cfg(feature = "yaml")]
#[cfg_attr(docsrs, doc(cfg(feature = "yaml")))]
pub use yaml::YamlExt;
