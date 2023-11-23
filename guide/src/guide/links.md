<!--
This file contains links that can be shared across pages. RustDoc links cannot currently
be used by mdBook directly. Links are stable on crates.io so we can centralize what is
required in this file, albeit manually.

REF: https://github.com/rust-lang/mdBook/issues/1356
REF: https://github.com/rust-lang/cargo/issues/739
REF: https://github.com/tag1consulting/goose/issues/320
-->

[`ChangeToken`]: https://docs.rs/more-changetoken/2.0.0/tokens/trait.ChangeToken.html

[`Value`]: https://docs.rs/more-config/2.0.0/config/type.Value.html

[`ConfigurationRoot`]: https://docs.rs/more-config/2.0.0/config/trait.ConfigurationRoot.html
[`Configuration`]: https://docs.rs/more-config/2.0.0/config/trait.Configuration.html
[`ConfigurationSection`]: https://docs.rs/more-config/2.0.0/config/trait.ConfigurationSection.html
[`section`]: https://docs.rs/more-config/2.0.0/config/trait.Configuration.html#method.section
[`children`]: https://docs.rs/more-config/2.0.0/config/trait.Configuration.html#method.children
[`exists`]: https://docs.rs/more-config/2.0.0/config/trait.ConfigurationSectionExtensions.html#method.exists

[`ConfigurationProvider`]: https://docs.rs/more-config/2.0.0/config/trait.ConfigurationProvider.html
[`ConfigurationProvider::reload_token`]: https://docs.rs/more-config/2.0.0/config/trait.ConfigurationProvider.html#method.reload_token

[`ConfigurationBinder`]: https://docs.rs/more-config/2.0.0/config/trait.ConfigurationBinder.html
[`bind`]: https://docs.rs/more-config/2.0.0/config/trait.ConfigurationBinder.html#method.bind
[`reify`]: https://docs.rs/more-config/2.0.0/config/trait.ConfigurationBinder.html#method.reify
[`get_value`]: https://docs.rs/more-config/2.0.0/config/trait.ConfigurationBinder.html#method.get_value
[`get_value_or_default`]: https://docs.rs/more-config/2.0.0/config/trait.ConfigurationBinder.html#method.get_value_or_default

[`CommandLineConfigurationSource`]: https://docs.rs/more-config/2.0.0/config/struct.CommandLineConfigurationSource.html
[`CommandLineConfigurationProvider`]: https://docs.rs/more-config/2.0.0/config/struct.CommandLineConfigurationProvider.html
[`add_command_line_map`]: https://docs.rs/more-config/2.0.0/config/trait.CommandLineConfigurationBuilderExtensions.html#method.add_command_line_map

[`EnvironmentVariablesConfigurationProvider`]: https://docs.rs/more-config/2.0.0/config/struct.EnvironmentVariablesConfigurationProvider.html
[`add_env_vars`]: https://docs.rs/more-config/2.0.0/config/struct.EnvironmentVariablesExtensions.html#method.add_env_vars
[`add_env_vars_with_prefix`]: https://docs.rs/more-config/2.0.0/config/struct.EnvironmentVariablesExtensions.html#method.add_env_vars_with_prefix

[`FileSource`]: https://docs.rs/more-config/2.0.0/config/struct.FileSource.html
[`path`]: https://docs.rs/more-config/2.0.0/config/struct.FileSource.html#method.path
[`optional`]: https://docs.rs/more-config/2.0.0/config/struct.FileSource.html#method.optional
[`reload_on_change`]: https://docs.rs/more-config/2.0.0/config/struct.FileSource.html#method.reload_on_change
[`reload_delay`]: https://docs.rs/more-config/2.0.0/config/struct.FileSource.html#method.reload_delay
[`FileSourceBuilder`]: https://docs.rs/more-config/2.0.0/config/struct.FileSourceBuilder.html
[`FileSourceBuilderExtensions`]: https://docs.rs/more-config/2.0.0/config/trait.FileSourceBuilderExtensions.html

[`JsonConfigurationProvider`]: https://docs.rs/more-config/2.0.0/config/struct.JsonConfigurationProvider.html
[`XmlConfigurationProvider`]: https://docs.rs/more-config/2.0.0/config/struct.XmlConfigurationProvider.html
[`IniConfigurationProvider`]: https://docs.rs/more-config/2.0.0/config/struct.IniConfigurationProvider.html
[`MemoryConfigurationProvider`]: https://docs.rs/more-config/2.0.0/config/struct.MemoryConfigurationProvider.html
