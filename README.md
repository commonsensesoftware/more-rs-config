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

- _default_ - Abstractions for configuration, including the **std** features
- **std** - Standard configuration implementation
- **mem** - An in-memory configuration source
- **env** - An environment variables configuration source
- **cmd** - A command-line argument configuration source
- **json** - A \*.json file configuration source
- **xml** - A \*.xml file configuration source
- **ini** - An \*.ini file configuration source
- **chained** - Chain multiple configuration sources
- **binder** - Bind a configuration to strongly-typed values and structs

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

The configuration can be loaded and accessed:

```rust
use config::{*, ext::*};

fn main() {
    let config = DefaultConfigurationBuilder::new()
        .add_json_file("demo.json")
        .build()
        .unwrap();
    
    if let Some(demo) = config.get("demo") {
      if *demo == "true" {
        println!("{}", config.get("text").unwrap());
        println!("{}", config.get("clients:0:region").unwrap());
        return;
      }
    }
    
    println!("Not a demo!");
}
```

Raw configuration values can be used, but they are much more interesting when we data bind them to strongly-typed values:

```rust
use serde::Deserialize;

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
```

```rust
use config::{*, ext::*};

fn main() {
    let config = DefaultConfigurationBuilder::new()
        .add_json_file("demo.json")
        .build()
        .unwrap();
    let app: AppOptions = config.reify();
    
    if app.demo {
        println!("{}", &config.text);
        println!("{}", config.clients[0].region);
        return;
    }
    
    println!("Not a demo!");
}
```

## License

This project is licensed under the [MIT license].

[MIT license]: https://github.com/commonsensesoftware/more-rs-config/blob/main/LICENSE