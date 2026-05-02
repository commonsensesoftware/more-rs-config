{{#include links.md}}

# In-Memory Configuration Provider

>These features are only available if the **mem** feature is activated

The [mem::Provider] uses an in-memory collection as configuration key-value pairs. This is most useful as a default
configuration or when providing test values.

The following code adds a memory collection to the configuration system and displays the settings:

```rust
use config::prelude::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error + 'static>> {
    let config = config::builder()
        .add_in_memory(&[
            ("MyKey", "Dictionary MyKey Value"),
            ("Position:Title", "Dictionary_Title"),
            ("Position:Name", "Dictionary_Name"),
            ("Logging:LogLevel:Default", "Warning"),
        ])
        .build()?;
    let my_key_value = config.get("MyKey").unwrap();
    let title = config.get("Position:Title").unwrap();
    let name = config.section("Position").get("Name").unwrap();
    let default_log_level = config.get("Logging:LogLevel:Default").unwrap();

    println!("MyKey value: {my_key_value}\n\
              Title: {title}\n\
              Name: {name}\n\
              Default Log Level: {default_log_level}");

    Ok(())
}
```