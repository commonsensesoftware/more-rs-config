<!--
This file contains links that can be shared across pages. RustDoc links cannot currently
be used by mdBook directly. Links are stable on crates.io so we can centralize what is
required in this file, albeit manually.

REF: https://github.com/rust-lang/mdBook/issues/1356
REF: https://github.com/rust-lang/cargo/issues/739
REF: https://github.com/tag1consulting/goose/issues/320
-->

[ChangeToken]: https://docs.rs/more-changetoken/2.1.0/tokens/trait.ChangeToken.html

[Error]: https://docs.rs/more-config/3.0.0/config/struct.Error.html
[Settings]: https://docs.rs/more-config/3.0.0/config/struct.Settings.html
[Configuration]: https://docs.rs/more-config/3.0.0/config/struct.Configuration.html
[Section]: https://docs.rs/more-config/3.0.0/config/struct.Section.html
[section]: https://docs.rs/more-config/3.0.0/config/struct.Configuration.html#method.section
[sections]: https://docs.rs/more-config/3.0.0/config/struct.Configuration.html#method.sections
[exists]: https://docs.rs/more-config/3.0.0/config/struct.Section.html#method.exists

[Provider]: https://docs.rs/more-config/3.0.0/config/trait.Provider.html
[Provider::reload_token]: https://docs.rs/more-config/3.0.0/config/trait.Provider.html#method.reload_token

[Binder]: https://docs.rs/more-config/3.0.0/config/prelude/trait.Binder.html
[bind]: https://docs.rs/more-config/3.0.0/config/prelude/trait.Binder.html#method.bind
[reify]: https://docs.rs/more-config/3.0.0/config/prelude/trait.Binder.html#method.reify
[get_value]: https://docs.rs/more-config/3.0.0/config/prelude/trait.Binder.html#method.get_value
[get_value_or_default]: https://docs.rs/more-config/3.0.0/config/prelude/trait.Binder.html#method.get_value_or_default

[cmd::Provider]: https://docs.rs/more-config/3.0.0/config/cmd/struct.Provider.html
[add_command_line_map]: https://docs.rs/more-config/3.0.0/config/prelude/trait.CommandLineExt.html#method.add_command_line_map

[env::Provider]: https://docs.rs/more-config/3.0.0/config/env/struct.Provider.html
[add_env_vars]: https://docs.rs/more-config/3.0.0/config/prelude/trait.EnvVarsExt.html#method.add_env_vars
[add_env_vars_with_prefix]: https://docs.rs/more-config/3.0.0/config/prelude/trait.EnvVarsExt.html#method.add_env_vars_with_prefix

[FileSource]: https://docs.rs/more-config/3.0.0/config/struct.FileSource.html
[path]: https://docs.rs/more-config/3.0.0/config/struct.FileSource.html#method.path
[optional]: https://docs.rs/more-config/3.0.0/config/struct.FileSource.html#method.optional
[reload_on_change]: https://docs.rs/more-config/3.0.0/config/struct.FileSource.html#method.reload_on_change
[FileSourceBuilder]: https://docs.rs/more-config/3.0.0/config/struct.FileSourceBuilder.html
[FileSourceBuilderExt]: https://docs.rs/more-config/3.0.0/config/prelude/trait.FileSourceBuilderExt.html

[ini::Provider]: https://docs.rs/more-config/3.0.0/config/ini/struct.Provider.html
[json::Provider]: https://docs.rs/more-config/3.0.0/config/json/struct.Provider.html
[mem::Provider]: https://docs.rs/more-config/3.0.0/config/mem/struct.Provider.html
[typed::Provider]: https://docs.rs/more-config/3.0.0/config/typed/struct.Provider.html
[xml::Provider]: https://docs.rs/more-config/3.0.0/config/xml/struct.Provider.html
[yaml::Provider]: https://docs.rs/more-config/3.0.0/config/yaml/struct.Provider.html
