{{#include links.md}}

# YAML Configuration Provider

>These features are only available if the **yaml** feature is activated

The [`YamlConfigurationProvider`] supports loading configuration from a `*.yaml` file.

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
use config::{*, ext::*};

fn main() {
    let config = DefaultConfigurationBuilder::new()
        .add_yaml_file("appsettings.yaml")
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
