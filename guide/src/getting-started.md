# Getting Started

The simplest way to get started is to install the crate using all features.

```bash
cargo add more-config --features all
```

This includes all features _except_ the **async** feature. The **async** feature intersects with all other features. If you would like all features with asynchronous support use:

```bash
cargo add more-config --features all,async
```

Once you know which configuration sources you want to support, you can limit the features to only the ones you need.

## Example

Configuration is a common requirement of virtually any application and can be performed using one or more configuration providers. Configuration providers read configuration data from key-value pairs using a variety of configuration sources:

- Settings files, such as `appsettings.json`
- Environment variables
- Command-line arguments
- In-memory data structures
- Custom providers

```rust
use config::{*, ext::*};

fn main() {
    let config = DefaultConfigurationBuilder::new()
        .add_in_memory(&[("MyKey", "MyValue")])
        .add_json_file("appsettings.json".is().optional())
        .add_env_vars()
        .add_command_line()
        .build()
        .unwrap();

    println!("MyKey = {}", config.get("MyKey").unwrap().as_str());
}
```

Configuration providers that are added later have higher priority and override previous key settings. For example, if `MyKey` is set in both `appsettings.json` and an environment variable, then the environment variable value is used. If `appsettings.json` does not exist or contain `MyKey` and there is no environment variable for `MyKey`, then the in-memory value of `MyValue` is used. Finally, if command-line argument `--MyKey` is provided, it overrides all other values.