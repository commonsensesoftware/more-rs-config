# Introduction

`more-config` is a crate containing all of the fundamental abstractions for configuration in Rust.

## Features

This crate provides the following features:

- _default_ - Abstractions for configuration, including the **cmd**, **env**, and **mem** features
- **all** - Includes all features, except **async**
- **async** - Use configuration in an asynchronous context
- **binder** - Bind a configuration to strongly-typed values and structs
- **chained** - Chain multiple configuration sources
- **cmd** - A command-line argument configuration source
- **env** - An environment variables configuration source
- **ini** - An \*.ini file configuration source
- **json** - A \*.json file configuration source
- **mem** - An in-memory configuration source
- **xml** - A \*.xml file configuration source

>Use `--features all,async` for all features with asynchronous support

## Contributing

`more-config` is free and open source. You can find the source code on [GitHub](https://github.com/commonsensesoftware/more-rs-config)
and issues and feature requests can be posted on the [GitHub issue tracker](https://github.com/commonsensesoftware/more-rs-config/issues).
`more-config` relies on the community to fix bugs and add features: if you'd like to contribute, please read the
[CONTRIBUTING](https://github.com/commonsensesoftware/more-rs-config/blob/main/CONTRIBUTING.md) guide and consider opening
a [pull request](https://github.com/commonsensesoftware/more-rs-config/pulls).

## License

This project is licensed under the [MIT](https://github.com/commonsensesoftware/more-rs-config/blob/main/LICENSE) license.