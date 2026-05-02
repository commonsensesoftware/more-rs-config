{{#include links.md}}

# INI Configuration Provider

>These features are only available if the **ini** feature is activated

The [ini::Provider] supports loading configuration from an `*.ini` file.

The following code adds several configuration providers, including a couple of `*.ini` files:

```rust
use config::prelude::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error + 'static>> {
    let name = std::env::var("ENVIRONMENT").or_else("production");
    let config = config::builder()
        .add_ini_file("MyIniConfig.ini".is().optional())
        .add_ini_file(format!("MyIniConfig.{name}.ini").is().optional())
        .add_env_vars()
        .add_command_line()
        .build()?;

    Ok(())
}
```

In the preceding code, settings in the `MyIniConfig.ini` and `MyIniConfig.{Environment}.ini` files are overridden by
settings in the:

- Environment variables configuration provider
- Command-line configuration provider

Assume the `MyIniConfig.ini` file contains:

```ini
MyKey="MyIniConfig.ini Value"

[Position]
Title="My INI Config title"
Name="My INI Config name"

[Logging:LogLevel]
Default=Information
App=Warning
```

The following code displays several of the preceding configurations settings:

```rust
let my_key_value = config.get("MyKey").unwrap();
let title = config.get("Position:Title").unwrap();
let name = config.section("Position").get("Name").unwrap();
let default_log_level = config.get("Logging:LogLevel:Default").unwrap();

println!("MyKey value: {my_key_value}\n\
          Title: {title}\n\
          Name: {name}\n\
          Default Log Level: {default_log_level}");
```
