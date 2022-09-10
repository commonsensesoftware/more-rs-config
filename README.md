# More Configuration Crate

[![Crates.io][crates-badge]][crates-url]
[![MIT licensed][mit-badge]][mit-url]

[crates-badge]: https://img.shields.io/crates/v/more-config.svg
[crates-url]: https://crates.io/crates/more-config
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/commonsensesoftware/more-rs-config/blob/main/LICENSE

This library contains all of the fundamental abstractions for defining configurations.

## Features

This crate provides the following features:

- _Default_ - Provides the abstractions for the configuration
- **std** - Provides the standard implementation for configuration
- **mem** - Provides an implementation for an in-memory configuration source
- **env** - Provides an implementation for an environment variables configuration source
- **cmd** - Provides an implementation for a command-line argument configuration source
- **ini** - Provides an implementation for an \*.ini file configuration source
- **json** - Provides an implementation for a \*.json file configuration source
- **chained** - Provides an implementation for chaining configuration sources
- **binder** - Provides extensions for binding a configuration to an options data structure

## License

This project is licensed under the [MIT license].

[MIT license]: https://github.com/commonsensesoftware/more-rs-config/blob/main/LICENSE