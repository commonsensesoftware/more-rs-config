{{#include links.md}}

# JSON Configuration Provider

>These features are only available if the **json** feature is activated

The [`JsonConfigurationProvider`] supports loading configuration from a `*.json` file.

Consider the following `appsettings.json` file:

```json
{
  "Position": {
    "Title": "Editor",
    "Name": "Joe Smith"
  },
  "MyKey": "My appsettings.json Value",
  "Logging": {
    "LogLevel": {
      "Default": "Information",
      "App": "Warning",
      "App.Hosting.Lifetime": "Information"
    }
  },
  "AllowedHosts": "*"
}
```

The following code displays several of the preceding configurations settings:

```rust
use config::{*, ext::*};

fn main() {
    let config = DefaultConfigurationBuilder::new()
        .add_json_file("appsettings.json")
        .build()
        .unwrap();

    let my_key_value = config.get("MyKey").unwrap();
    let title = config.get("Position:Title").unwrap();
    let name = config.section("Position").get("Name").unwrap();
    let default_log_level = config.get("Logging:LogLevel:Default").unwrap();

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