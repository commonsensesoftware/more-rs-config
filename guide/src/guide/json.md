{{#include links.md}}

# JSON Configuration Provider

>These features are only available if the **json** feature is activated

The [json::Provider] supports loading configuration from a `*.json` file.

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
use config::prelude::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error + 'static>> {
    let config = config::builder().add_json_file("appsettings.json").build()?;
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