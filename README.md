# More Configuration &emsp; ![CI][ci-badge] [![Crates.io][crates-badge]][crates-url] [![MIT licensed][mit-badge]][mit-url]

[crates-badge]: https://img.shields.io/crates/v/more-config.svg
[crates-url]: https://crates.io/crates/more-config
[mit-badge]: https://img.shields.io/badge/license-MIT-blueviolet.svg
[mit-url]: https://github.com/commonsensesoftware/more-rs-config/blob/main/LICENSE
[ci-badge]: https://github.com/commonsensesoftware/more-rs-config/actions/workflows/ci.yml/badge.svg

More Configuration is a configuration library for Rust.

You may be looking for:

- [User Guide](https://commonsensesoftware.github.io/more-rs-config)
- [API Documentation](https://docs.rs/more-config)
- [Release Notes](https://github.com/commonsensesoftware/more-rs-config/releases)

## Features

This crate provides the following features:

- _default_ - Abstractions for configuration, including the **cmd**, **env**, and **mem** features
- **all** - Includes all features, except **async**
- **async** - Use configuration in an asynchronous context
- **binder** - Bind a configuration to strongly-typed values and structs
- **chained** - Chain multiple configuration providers
- **cmd** - Configuration provided by command-line arguments
- **env** - Configuration provided by environment variables
- **ini** - Configuration provided by an \*.ini file
- **json** - Configuration provided by a \*.json file
- **mem** - Configuration provided by in-memory data
- **typed** - Configuration provided by strongly-typed, in-memory data
- **xml** - Configuration provided by a \*.xml file
- **yaml** - Configuration provided by a \*.yaml file

>Use `--features all,async` for all features with asynchronous support

## Configuration in Action

Consider the following `demo.json` file:

```json
{
  "text": "Hello world!",
  "demo": true,
  "clients": [{
    "region": "us-west",
    "url": "https://tempuri.org"
  }]
}
```

The configuration can be loaded, merged, and accessed from multiple sources:

```rust,no_run
use config::prelude::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error + 'static>> {
    let config = config::builder()
        .add_in_memory(&[("Demo", "false")])
        .add_json_file("demo.json".is().optional())
        .add_env_vars()
        .add_command_line()
        .build()?;
    
    if let Some(demo) = config.get("demo") {
      if demo == "true" {
        println!("{}", config.get("Text").unwrap_or_default());
        println!("{}", config.get("Clients:0:Region").unwrap_or_default());
      }
    } else {
      println!("Not a demo!");
    }
    
    Ok(())
}
```

Raw configuration values can be used, but they are much more interesting when we data bind them to strongly-typed values.

>The first letter of JSON configuration keys are normalized to uppercase.

```rust,no_run
use config::prelude::*;
use serde::Deserialize;
use std::{error::Error, path::Path};

#[derive(Default, Deserialize)]
struct Client {
  region: String,
  url: String,
}

#[derive(Default, Deserialize)]
struct AppOptions {
  text: String,
  demo: bool,
  clients: Vec<Client>,
}

fn main() -> Result<(), Box<dyn Error + 'static>> {
  let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("demo.json");
  let config = config::builder().add_json_file(path).build()?;
  let app: AppOptions = config.reify()?;

  if app.demo {
    println!("{}", &app.text);
    println!("{}", &app.clients[0].region);
  } else {
    println!("Not a demo!");
  }
}
```

## Examples

A simple demonstration application is provided that combines in-memory settings, a demo.json file, and allows command
line arguments. Run it with:

```bash
cargo run --example demo
```

To highlight overriding configuration via the command line, run it with:

```bash
cargo run --example demo -- --text "I'm a teapot!"
```

## Minimum Supported Rust Version

When increasing the minimum supported Rust version (MSRV), the new version must have been released at least six months
ago. The current MSRV is 1.60.

## License

This project is licensed under the [MIT license].

[MIT license]: https://github.com/commonsensesoftware/more-rs-config/blob/main/LICENSE