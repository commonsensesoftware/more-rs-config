{{#include links.md}}

# Working With Files

A [Provider](abstractions.md#configuration-provider) that is based on a file should support a [FileSource]:

```rust
pub struct FileSource {
    pub path: PathBuf,
    pub optional: bool,
    pub reload_on_change: bool,
}
```

An [optional] file means that the [path] does not need to exist. When [reload_on_change] is specified, the provider will
watch for changes to [path] and trigger a notification via [Provider::reload_token]. A file change might trigger before
a file has been completely written, which is operating system dependent.

All of the built-in, file-based configuration providers support accepting a [FileSource]. A file source is most commonly
just a file path, but it may include additional configuration features. The [FileSourceBuilder] struct and
[FileSourceBuilderExt] trait provide several methods of specifying a [FileSource] and its options in a fluent manner.

```rust
use config::{FileSource, prelude::*};
use std::error::Error;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn Error + 'static>> {
    let xml = PathBuf::from("settings.xml");
    let config = config::builder()
        .add_ini_file(FileSource::new(PathBuf::("prod.cfg.ini"), false, false))
        .add_ini_file(FileSource::optional(PathBuf::("dev.cfg.ini")))
        .add_xml_file(xml.is().optional())
        .add_json_file("settings.json".is().optional().reloadable())
        .build()?;

    for (key, value) in &config {
        println!("{key} = {value}");
    }

    Ok(())
}
```