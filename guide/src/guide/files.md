{{#include links.md}}

# Working With Files

A [`ConfigurationProvider`](abstractions.md#configuration-provider) that is based on a file should support a `FileSource`:

```rust
pub struct FileSource {
    pub path: PathBuf,
    pub optional: bool,
    pub reload_on_change: bool,
    pub reload_delay: Duration,
}
```

An [`optional`] file means that the [`path`] does not need to exist. When [`reload_on_change`] is specified, the provider will watch for changes to [`path`] and trigger a notification via [`ConfigurationProvider::reload_token`]. A file change might trigger before a file has been completely written, which is operating system dependent. [`reload_delay`] indicates how long a provider should wait to reload when a change is detected. The default duration is 250 milliseconds.

All of the built-in, file-based configuration providers support accepting a [`FileSource`]. A file source is most commonly just a file path, but it may include additional configuration features. The [`FileSourceBuilder`] struct and [`FileSourceBuilderExtensions`] trait provide several methods of specifying a [`FileSource`] and its options in a fluent manner.

```rust
use config::{*, ext::*};
use std::path::PathBuf;

fn main() {
    let xml = PathBuf::from("settings.xml");
    let config = DefaultConfigurationBuilder::new()
        .add_ini_file(FileSource::new(PathBuf::("prod.cfg.ini"), false, false, None))
        .add_ini_file(FileSource::optional(PathBuf::("dev.cfg.ini")))
        .add_xml_file(xml.is().optional())
        .add_json_file("settings.json".is().optional().reloadable())
        .build()
        .unwrap();

    for (key, value) in config.iter() {
        println!("{} = {}", key, value);
    }
}
```