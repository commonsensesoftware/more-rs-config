{{#include links.md}}

# Data Binding

>Data binding requires the **binder** feature, which will also trigger activation of the optional **serde** dependency
and is required for deserialization.

Data binding leverages the [serde](https://crates.io/crates/serde) crate to enable deserializing configurations in part,
or in whole, into strongly-typed structures. It is also possible to retrieve strongly-typed scalar values.

A [Configuration](abstractions.md#configuration) is deserialized through the [Binder] trait:

```rust
pub trait Binder {
    fn reify<T>(&self) -> config::Result<T>
    where
        T: DeserializeOwned;

    fn bind<T>(&self, instance: &mut T) -> config::Result
    where
        T: DeserializeOwned;

    fn bind_at<T>(&self, key: impl AsRef<str>, instance: &mut T) -> config::Result
    where
        T: DeserializeOwned;

    fn get_value<T>(&self, key: impl AsRef<str>) -> Result<Option<T>, T::Err>
    where
        T: FromStr;
    
    fn get_value_or_default<T>(&self, key: impl AsRef<str>) -> Result<T, T::Err>
    where
        T: FromStr + Default;
}
```

Consider the following struct:

```rust
use serde::Deserialize;

#[derive(Default, Deserialize)]
struct ContactOptions {
    name: String,
    primary: bool,
    phones: Vec<String>,
}
```

>Configuration keys are normalized or expected to otherwise be Pascal Case for consistency.

The following demonstrates how to load a configuration and then reify the configuration into the struct that was
defined above. This example used the [in-memory configuration provider](memory.md), but any configuration provider or
multiple configuration providers can be used.

```rust
use config::prelude::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error + 'static>> {
    let config = config::builder()
        .add_in_memory(&[
            ("name", "John Doe"),
            ("primary", "true"),
            ("phones:0", "+44 1234567"),
            ("phones:1", "+44 2345678"),
        ])
        .build()?;
    let primary: bool = config.get_value_or_default("primary")?;
    let options: ContactOptions = config.reify()?;

    println!("Is Primary: {primary}");
    println!("{}", &options.name);
    println!("Phones:");

    for phone in &contact.phones {
        println!("\n  {phone}");
    }

    Ok(())
}
```

It is also possible to bind an existing structure to an entire [configuration](abstractions.md#configuration) or bind at
a specific [configuration section](abstractions.md#configuration-section).

```rust
use config::prelude::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error + 'static>> {
    let config = config::builder()
        .add_in_memory(&[
            ("name", "John Doe"),
            ("primary", "true"),
            ("phones:0", "+44 1234567"),
            ("phones:1", "+44 2345678"),
        ])
        .build()?;
    let mut options = ContactOptions::default();

    config.bind(&mut options)?;

    println!("{}", &options.name);
    println!("Phones:");

    for phone in &contact.phones {
        println!("\n  {phone}");
    }

    Ok(())
}
```

>**Note**: The bound struct must implement `Deserialize::deserialize_in_place` to perform a true, in-place update. The
default implementation creates a new struct and binds to it, which is essentially the same as mutating the struct to
the result of [reify].

## Bind an Array

[bind] supports binding arrays to objects using array indices in configuration keys.

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

```rust
use config::prelude::*;
use serde::Deserialize;
use std::error::Error;

#[derive(Default, Deserialize)]
struct ArrayExample {
    entries: Vec<String>,
}

fn main() -> Result<(), Box<dyn Error + 'static>> {
    let config = DefaultConfigurationBuilder::new()
        .add_json_file("MyArray.json")
        .build()?;
    let array: ArrayExample = config.reify()?;

    for (i, item) in array.entries.iter().enumerate() {
        println!("Index: {i}, Value: {item}");
    }

    Ok(())
}
```

The preceding code returns the following output. Note that index `3` has the value `value40`, which corresponds to
`"4": "value40"` in `MyArray.json`. The bound array indices are continuous and not bound to the configuration key index.
The configuration binder isn't capable of binding null values or creating null entries in bound objects; however, a
missing value can be mapped to `Option`.

```text
Index: 0  Value: value00
Index: 1  Value: value10
Index: 2  Value: value20
Index: 3  Value: value40
Index: 4  Value: value50
```