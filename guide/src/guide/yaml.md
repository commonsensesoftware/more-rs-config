{{#include links.md}}

# YAML Configuration Provider

>These features are only available if the **yaml** feature is activated

The [yaml::Provider] supports loading configuration from a `*.yaml` file.

Consider the following `appsettings.yaml` file:

```yaml
Position:
  Title: Editor
  Name: Joe Smith
MyKey: My appsettings.yaml Value
Logging:
  LogLevel:
    Default: Information
    App: Warning
    App.Hosting.Lifetime: Information
AllowedHosts: "*"
```

The following code displays several of the preceding configurations settings:

```rust
use config::prelude::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error + 'static>> {
    let config = config::builder().add_yaml_file("appsettings.yaml").build()?;
    let my_key_value = config.get("MyKey");
    let title = config.get("Position:Title");
    let name = config.section("Position").get("Name");
    let default_log_level = config.get("Logging:LogLevel:Default");

    println!("MyKey value: {my_key_value}\n\
              Title: {title}\n\
              Name: {name}\n\
              Default Log Level: {default_log_level}");

    Ok(())
}