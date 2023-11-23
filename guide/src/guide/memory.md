{{#include links.md}}

# In-Memory Configuration Provider

>These features are only available if the **mem** feature is activated

The [`MemoryConfigurationProvider`] uses an in-memory collection as configuration key-value pairs. This is most useful as a default configuration or when providing test values.

The following code adds a memory collection to the configuration system and displays the settings:

```rust
use config::{*, ext::*};

fn main() {
    let config = DefaultConfigurationBuilder::new()
        .add_in_memory(&[
            ("MyKey", "Dictionary MyKey Value"),
            ("Position:Title", "Dictionary_Title"),
            ("Position:Name", "Dictionary_Name"),
            ("Logging:LogLevel:Default", "Warning"),
        ])
        .build()
        .unwrap();

    let my_key_value = config.get("MyKey").unwrap().as_str();
    let title = config.get("Position:Title").unwrap().as_str();
    let name = config.section("Position").get("Name").unwrap().as_str();
    let default_log_level = config.get("Logging:LogLevel:Default").unwrap().as_str();

    println!("MyKey value: {}\n\
              Title: {}\n\
              Name: {}\n\
              Default Log Level: {}",
              my_key_value,
              title,
              name,
              default_log_level);
}
```