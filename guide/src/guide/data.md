{{#include links.md}}

# Working With Configuration Data

There are several different ways to work with configuration data. [Configuration sources](abstractions.md#configuration-source) are normalized to a generic key-value pair format, which can then be merged and consumed universally; regardless of the original format.

## Hierarchical Configuration Data

The Configuration API reads hierarchical configuration data by flattening the hierarchical data with the use of a delimiter in the configuration keys.

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
use config::{*, ext::*};

fn main() {
    let config = DefaultConfigurationBuilder::new()
        .add_json_file("appsettings.json")
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
```

The preferred way to read hierarchical configuration data is using the _Options_ pattern provided by the [more-options](https://crates.io/crates/more-options) crate. The [`section`] and [`children`] methods are available to isolate sections and children of a section in the configuration data.

## Configuration Keys and Values

Configuration keys:

- Are case-insensitive; for example, `ConnectionString` and `connectionstring` are treated as equivalent keys.
- If a key and value is set in more than one [configuration providers](abstractions.md#configuration-provider), the value from the last provider added is used.
- Hierarchical keys
  - Within the Configuration API, a colon separator (`:`) works on all platforms.
  - In environment variables, a colon separator may not work on all platforms. A double underscore, `__`, is supported by all platforms and is automatically converted into a colon `:`.
- The [`ConfigurationBinder`](binding.md) supports binding arrays to objects using array indices in configuration keys.

Configuration values:

- Are strings
- Null values can't be stored in configuration or bound to objects

## Get Value

The [`get_value`] and [`get_value_or_default`] methods extract a single value from configuration with a specified key and converts it to the specified type.

```rust
use config::{*, ext::*};

fn main() {
    let config = DefaultConfigurationBuilder::new()
        .add_json_file("settings.json")
        .build()
        .unwrap();

    let number: Option<u8> = config.get_value("NumberKey").unwrap().unwrap_or(99);
    let flag: bool = config.get_value_or_default("Enabled").unwrap();

    println!("Number = {}", number);
    println!("Flag = {}", flag);
}
```

In the preceding code, if `NumberKey` isn't found in the configuration, the default value of `99` is used. If `Enabled` isn't found in the configuration, it will default to `false`, which is the `Default::default()` for `bool`.

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

[`section`] returns a configuration subsection with the specified subsection key.

The following code returns values for `section1`:

```rust
let section = config.section("section1");

println!("section1:key0: {}\n\
          section1:key1: {}",
          section.get("key0").unwrap().as_str(),
          section.get("key1").unwrap().as_str());
```

The following code returns values for `section2:subsection0`:

```rust
let section = config.section("section2:subsection0");

println!("section2:subsection0:key0: {}\n\
          section2:subsection0:key0: {}",
          section.get("key0").unwrap().as_str(),
          section.get("key1").unwrap().as_str());
```

If a matching section isn't found, an empty [`ConfigurationSection`] is returned.

### Children and Exists

The following code calls [`children`] and returns values for `section2:subsection0`:

```rust
let section = config.section("section2");

if section.exists() {
  for subsection in section.children() {
    let key1 = format!("{}:key0", section.key());
    let key2 = format!("{}:key1", section.key());
    
    println!("{} value: {}\n\
              {} value: {}",
              &key1,
              &key2,
              section.get(&key1).unwrap().as_str(),
              section.get(&key2).unwrap().as_str());
  }
} else {
  println!("section2 does not exist.");
}

println!("section1:key0: {}\n\
          section1:key1: {}",
          section.get("key0").unwrap().as_str(),
          section.get("key1").unwrap().as_str());
```

The preceding code uses the [`exists`] extension to verify the section exists.

