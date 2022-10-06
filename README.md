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

## Configuration

Configuration is a common requirement of virtually any application and can be performed using
one or more [configuration providers](#configuration-providers). Configuration providers read
configuration data from key-value pairs using a variety of configuration sources:

- Settings files, such as `appsettings.json`
- Environment variables
- Command-line arguments
- In-memory data structures
- Custom providers

```rust
fn main() {
    let source = MemoryConfigurationSource::new(
        [("MyKey", "MyValue")]
            .iter()
            .map(|t| (t.0.to_owned(), t.1.to_owned()))
            .collect(),
    );
    let mut builder = DefaultConfigurationBuilder::new();

    builder.add(Box::new(source));

    let root = builder.build();

    for provider in root.providers() {
        println!("{}", provider.name());
    }
}
```

Configuration providers that are added later have higher priority and override previous key settings. For example, if `MyKey` is set in both `appsettings.json` and the environment, the environment value is used. Using the default configuration providers, the [Command-line configuration provider]() overrides all other providers.

## JSON

Consider the following `appsettings.json` file:

```json
{
  "Position": {
    "Title": "Editor",
    "Name": "Joe Smith"
  },
  "MyKey": "My appsettings.json Value",
  "Logging": {
    "LogLevel": {
      "Default": "Information",
      "App": "Warning",
      "App.Hosting.Lifetime": "Information"
    }
  },
  "AllowedHosts": "*"
}
```

The following code displays several of the preceding configurations settings:

```rust
fn main() {
    let path = PathBuf::from("./appsettings.json");
    let config = DefaultConfigurationBuilder::new()
        .add_json_file(&path)
        .build();

    let my_key_value = config.get("MyKey").unwrap();
    let title = config.get("Position:Title").unwrap();
    let name = config.get("Position:Name").unwrap();
    let default_log_level = config.get("Logging:LogLevel:Default").unwrap();

    println!("MyKey value: {}\n\
              Title: {}\n\
              Name: {}\n\
              Default Log Level: {}",
              my_key_value,
              title,
              name,
              default_log_level);
}
```

## Environment Variables

The `EnvironmentVariablesConfigurationProvider` loads configuration from environment
variable key-value pairs.

The `:` separator doesn't work with environment variable hierarchical keys on all platforms. `__`,
the double underscore, is:

- Supported by all platforms; for example, the `:` separator is not supported by Bash, but `__` is.
- Automatically replaced by a `:`

```bash
export MyKey="My key from Environment"
export Position__Title=Console
export Position__Name="John Doe"
```

Call `add_env_vars` to add environment variables or `add_env_vars_with_prefix` with a
string to specify a prefix for environment variables:

```rust
let config = DefaultConfigurationBuilder::new()
        .add_env_vars_with_prefix("MyCustomPrefix_")
        .build();
```

Environment variables set with the `MyCustomPrefix_` prefix override the default configuration
providers. This includes environment variables without the prefix. The prefix is stripped off
when the configuration key-value pairs are read.

```bash
export MyCustomPrefix_MyKey="My key with MyCustomPrefix_ Environment"
export MyCustomPrefix_Position__Title="Custom Editor"
export MyCustomPrefix_Position__Name="Jane Doe"
```

### Naming of environment variables

Environment variable names reflect the structure of an `appsettings.json` file. Each element
in the hierarchy is separated by a double underscore. When the element structure includes an
array, the array index should be treated as an additional element name in this path. Consider
the following appsettings.json file and its equivalent values represented as environment variables.

```json
{
    "SmtpServer": "smtp.example.com",
    "Logging":
    [
        {
            "Name": "ToEmail",
            "Level": "Critical",
            "Args":
            {
                "FromAddress": "MySystem@example.com",
                "ToAddress": "SRE@example.com"
            }
        },
        {
            "Name": "ToConsole",
            "Level": "Information"
        }
    ]
}
```

```bash
export SmtpServer=smtp.example.com
export Logging__0__Name=ToEmail
export Logging__0__Level=Critical
export Logging__0__Args__FromAddress=MySystem@example.com
export Logging__0__Args__ToAddress=SRE@example.com
export Logging__1__Name=ToConsole
export Logging__1__Level=Information
```

## Command-line

The `CommandLineConfigurationProvider` loads configuration from command-line argument
key-value pairs. Configuration values set on the command-line can be used to override
configuration values set with all the other configuration providers.

### Command-line arguments

The following command sets keys and values using `=`:

```bash
myapp MyKey="Using =" Position:Title=Cmd Position:Name=Cmd_Joe
```

The following command sets keys and values using `/`:


```bash
myapp /MyKey "Using /" /Position:Title=Cmd /Position:Name=Cmd_Joe
```

The following command sets keys and values using `--`:

```bash
myapp --MyKey "Using --" --Position:Title=Cmd --Position:Name=Cmd_Joe
```

The key value:

- Must follow `=`, or the key must have a prefix of `--` or `/` when the value follows a space.
- Isn't required if `=` is used; for example, `MySetting=`.

Within the same command, don't mix command-line argument key-value pairs that use `=` with
key-value pairs that use a space.

### Switch mappings

Switch mappings allow key name replacement logic. Provide a hash map of switch replacements to the `add_command_line_map` method.

When the switch mappings hash map is used, the hash map is checked for a key that matches the
key provided by a command-line argument. If the command-line key is found in the hash map, the
hash map value is passed back to set the key-value pair into the application's configuration. A
switch mapping is required for any command-line key prefixed with a single dash (`-`).

Switch mappings hash map key rules:

- Switches must start with `-` or `--`.
- The switch mappings hash map must not contain duplicate keys.

To use a switch mappings hash map, pass it into the call to `add_command_line_map`:

```rust
let switch_mappings: HashMap<_, _> = vec![
            ("-k1", "key1"),
            ("-k2", "key2"),
            ("--alt3", "key3"),
            ("--alt4", "key4"),
            ("--alt5", "key5"),
            ("--alt6", "key6"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect();
let config = DefaultConfigurationBuilder::new()
        .add_command_line_map(switch_mappings)
        .build();
```

Run the following command works to test key replacement:

```bash
myapp -k1 value1 -k2 value2 --alt3=value2 /alt4=value3 --alt5 value5 /alt6 value6
```

The following code shows the key values for the replaced keys:

```rust
println!("Key1: {}\n\
          Key2: {}\n\
          Key3: {}\n\
          Key4: {}\n\
          Key5: {}\n\
          Key6: {}",
          config.get("Key1").unwrap(),
          config.get("Key2").unwrap(),
          config.get("Key3").unwrap(),
          config.get("Key4").unwrap(),
          config.get("Key5").unwrap(),
          config.get("Key6").unwrap());
```

## Hierarchical configuration data

The Configuration API reads hierarchical configuration data by flattening the hierarchical
data with the use of a delimiter in the configuration keys.

Consider the following `appsettings.json` file:

```json
{
  "Position": {
    "Title": "Editor",
    "Name": "Joe Smith"
  },
  "MyKey": "My appsettings.json Value",
  "Logging": {
    "LogLevel": {
      "Default": "Information",
      "App": "Warning",
      "App.Hosting.Lifetime": "Information"
    }
  },
  "AllowedHosts": "*"
}
```

The following code displays several of the configurations settings:

```rust
fn main() {
    let path = PathBuf::from("./appsettings.json");
    let config = DefaultConfigurationBuilder::new()
        .add_json_file(&path)
        .build();

    let my_key_value = config.get("MyKey").unwrap();
    let title = config.get("Position:Title").unwrap();
    let name = config.get("Position:Name").unwrap();
    let default_log_level = config.get("Logging:LogLevel:Default").unwrap();

    println!("MyKey value: {}\n\
              Title: {}\n\
              Name: {}\n\
              Default Log Level: {}",
              my_key_value,
              title,
              name,
              default_log_level);
}
```

The preferred way to read hierarchical configuration data is using the _Options_ pattern.

The `section` and `children` methods are available to isolate sections and
children of a section in the configuration data.

## Configuration keys and values

Configuration keys:

- Are case-insensitive; for example, `ConnectionString` and `connectionstring` are treated as equivalent keys.
- If a key and value is set in more than one configuration providers, the value from the last provider added is used.
- Hierarchical keys
  - Within the Configuration API, a colon separator (`:`) works on all platforms.
  - In environment variables, a colon separator may not work on all platforms. A double underscore, `__`, is supported by all platforms and is automatically converted into a colon `:`.
- The `ConfigurationBinder` supports binding arrays to objects using array indices in configuration keys.

Configuration values:

- Are strings.
- Null values can't be stored in configuration or bound to objects.

## Configuration providers

| Provider | Provides configuration from |
| -------- | --------------------------- |
| [Command-line configuration provider]() | Command-line parameters |
| [Custom configuration provider]() | Custom source |
| [Environment variables configuration provider]() | Enviroment variables |
| [JSON configuration provider]() | JSON files |
| [INI configuration provider]() | INI files |
| [Memory configuration provider]() | In-memory collection |

Configuration sources are read in the order that their configuration providers
are specified. Order configuration providers in code to suit the priorities for
the underlying configuration sources that the application requires.

A typical sequence of configuration providers is:

- `appsettings.json`
- `appsettings.{Environment}.json`
- Environment variables
- Command-line arguments

A common practice is to add the command-line configuration provider last in a
series of providers to allow command-line arguments to override configuration
set by the other providers.

## INI configuration provider

The `IniConfigurationProvider` loads configuration from INI file key-value pairs
at runtime.

The following code clears all the configuration providers and adds several configuration providers:

```rust
fn main() {
    let name = env::var("ENVIRONMENT").or_else("production");
    let config = DefaultConfigurationBuilder::new()
        .add_optional_ini_file(&PathBuf::from("MyIniConfig.ini"))
        .add_optional_ini_file(&PathBuf::from(format!("MyIniConfig.{}.ini", name)))
        .add_env_vars()
        .add_command_line(env::args().collect())
        .build();
}
```

In the preceding code, settings in the `MyIniConfig.ini` and
`MyIniConfig.{Environment}.ini` files are overridden by settings in the:

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
let name = config.get("Position:Name").unwrap();
let default_log_level = config.get("Logging:LogLevel:Default").unwrap();

println!("MyKey value: {}\n\
          Title: {}\n\
          Name: {}\n\
          Default Log Level: {}",
          my_key_value,
          title,
          name,
          default_log_level);
```

## JSON configuration provider

The `JsonConfigurationProvider` loads configuration from JSON file key-value pairs.

Consider the following code:

```rust
fn main() {
    let config = DefaultConfigurationBuilder::new()
        .add_optional_json_file(&PathBuf::from("MyIniConfig.json"))
        .build();
}
```

The preceding code configures the JSON configuration provider to load the `MyConfig.json`
file, if the file exists.

## Memory configuration provider

The `MemoryConfigurationProvider` uses an in-memory collection as configuration key-value pairs.

The following code adds a memory collection to the configuration system and displays the settings:

```rust
fn main() {
    let config = DefaultConfigurationBuilder::new()
        .add_in_memory(
          [
            ("MyKey", "Dictionary MyKey Value"),
            ("Position:Title", "Dictionary_Title"),
            ("Position:Name", "Dictionary_Name"),
            ("Logging:LogLevel:Default", "Warning"),
          ]
          .iter()
          .map(|t| (t.0.to_owned(), t.1.to_owned()))
          .collect()
        )
        .build();

    let my_key_value = config.get("MyKey").unwrap();
    let title = config.get("Position:Title").unwrap();
    let name = config.get("Position:Name").unwrap();
    let default_log_level = config.get("Logging:LogLevel:Default").unwrap();

    println!("MyKey value: {}\n\
              Title: {}\n\
              Name: {}\n\
              Default Log Level: {}",
              my_key_value,
              title,
              name,
              default_log_level);
}
```

## GetValue

The `ConfigurationBinder` `get_value` and `get_value_or_default` methods extract a
single value from configuration with a specified key and converts it to the
specified type.

```rust
fn main() {
    let config = DefaultConfigurationBuilder::new()
        .add_json_file(&PathBuf::from("settings.json"))
        .build();
    let number = config.get_value::<u8>("NumberKey").ok().or_else(99);
    let flag = config.get_value_or_default::<bool>("Enabled").ok();
}
```

In the preceding code, if `NumberKey` isn't found in the configuration, the default value
of `99` is used. If `Enabled` isn't found in the configuration, it will default to `false`,
which is the `Default::default()` for `bool`.

## Section, Children, and Exists

For the examples that follow, consider the following `MySubsection.json` file:

```json
{
  "section0": {
    "key0": "value00",
    "key1": "value01"
  },
  "section1": {
    "key0": "value10",
    "key1": "value11"
  },
  "section2": {
    "subsection0": {
      "key0": "value200",
      "key1": "value201"
    },
    "subsection1": {
      "key0": "value210",
      "key1": "value211"
    }
  }
}
```

### Section

`Configuration.section` returns a configuration subsection with the specified subsection key.

The following code returns values for `section1`:

```rust
let section = config.section("section1");
println!("section1:key0: {}\n\
          section1:key1: {}",
          section.get("key0").unwrap(),
          section.get("key1").unwrap());
```

The following code returns values for `section2:subsection0`:

```rust
let section = config.section("section2:subsection0");
println!("section2:subsection0:key0: {}\n\
          section2:subsection0:key0: {}",
          section.get("key0").unwrap(),
          section.get("key1").unwrap());
```

If a matching section isn't found, an empty `ConfigurationSection` is returned.

### Children and Exists

The following code calls `Configuration.children` and returns values for
`section2:subsection0`:

```rust
let section = config.section("section2");

if section.exists() {
  for subsection in &section.children() {
    let key1 = format!("{}:key0", section.key());
    let key2 = format!("{}:key1", section.key());
    println!("{} value: {}\n\
              {} value: {}",
              &key1,
              &key2,
              section.get(&key1).unwrap(),
              section.get(&key2).unwrap());
  }
} else {
  println!("section2 does not exist.");
}

println!("section1:key0: {}\n\
          section1:key1: {}",
          section.get("key0").unwrap(),
          section.get("key1").unwrap());
```

The preceding code uses the `ConfigurationSectionExtensions.exists` extension to
verify the section exists.

## Bind an array

`ConfigurationBinder.bind` supports binding arrays to objects using array indices
in configuration keys.

Consider `MyArray.json`:

```json
{
  "array": {
    "entries": {
      "0": "value00",
      "1": "value10",
      "2": "value20",
      "4": "value40",
      "5": "value50"
    }
  }
}
```

The following code reads the configuration and displays the values:

>Note that activating the **binder** feature will also trigger activation
>of the optional **serde** dependency, which is required for deserializaiton.

```rust
#[derive(Default, Deserialize)]
struct ArrayExample {
    entries: Vec<String>,
}

fn main() {
    let config = DefaultConfigurationBuilder::new()
        .add_json_file(&PathBuf::from("MyArray.json"))
        .build();
    let array: ArrayExample = config.reify();

    for (i, item) in array.iter().enumerate() {
      println!("Index: {}, Value: {}", i, item );
    }
}
```

The preceding code returns the following output. Note that index 3 has the
value `value40`, which corresponds to `"4": "value40"` in `MyArray.json`.
The bound array indices are continuous and not bound to the configuration
key index. The configuration binder isn't capable of binding null values
or creating null entries in bound objects.

```text
Index: 0  Value: value00
Index: 1  Value: value10
Index: 2  Value: value20
Index: 3  Value: value40
Index: 4  Value: value50
```

## License

This project is licensed under the [MIT license].

[MIT license]: https://github.com/commonsensesoftware/more-rs-config/blob/main/LICENSE