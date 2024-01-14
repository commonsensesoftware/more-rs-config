{{#include links.md}}

# Data-Structure Configuration Provider

>These features are only available if the **struct** feature is activated

The [`StructConfigurationProvider`] uses an in-memory data structure as configuration key-value pairs. This is most useful as an alternative default configuration to in-memory configuration or when providing test values.

The following code adds a data structure collection to the configuration system and displays the settings:

```rust
use config::{*, ext::*};
use serde::Serialize;

#[derive(Default, Serialize, Clone)]
#[serde(rename_all(serialize = "PascalCase"))]
struct PerfSettings {
    cores: u8,
} 

#[derive(Default, Serialize, Clone)]
#[serde(rename_all(serialize = "PascalCase"))]
struct AppOptions {
    title: String,
    perf: PerfSettings,
}

fn main() {
    let default = AppOptions {
        title: String::from("Banana processor"),
        perf: SubOptions{ cores: 7 },
    };

    let config = DefaultConfigurationBuilder::new()
        .add_struct(default.clone())
        .build()
        .unwrap();

    let title = config.get("title").unwrap().as_str();
    let cores = config.get("Perf:Cores").unwrap().as_str();

    println!("Title: {}\n\
              Cores: {}\n\
              title,
              cores);
}
```

Instead of a data structure, one can also load a tuple, a vector, or a map.
Values from Tuples and Vectors can be retrieved with their index as key:

```rust
use config::{*, ext::*};

fn main() {
    let value = std::vec::Vec::from([32, 56]);
    let config = DefaultConfigurationBuilder::new()
        .add_struct(value)
        .build()
        .unwrap();
    println!("[{}, {}]", config.get("0"), config.get("1"));
}
```

Or, same example with a tuple:

```rust
use config::{*, ext::*};

fn main() {
    let value = (32, 56);
    let config = DefaultConfigurationBuilder::new()
        .add_struct(value)
        .build()
        .unwrap();
    println!("[{}, {}]", config.get("0"), config.get("1"));
}
```

When loading maps, it is best to restrict to maps using strings as keys. When using strings as keys, it is possible to bind the data to some data structure:

```rust
use config::{*, ext::*};

#[derive(Deserialize)]
#[serde(rename_all(deserialize = "PascalCase"))]
struct FooBar {
    foo: String,
    karoucho: i32,
}

fn main() {
    let mut value: HashMap<&str, &str> = HashMap::new();
    value.insert("foo", "bar");
    value.insert("karoucho", "34");

    let config = DefaultConfigurationBuilder::new()
        .add_struct(value)
        .build()
        .unwrap();
    let foo = config.get("foo");
    println!("foo by query: {}", foo);
    let karoucho = config.get("karoucho");
    println!("karoucho by query: {}", karoucho);

    let options: FooBar = config.reify();
    println!("foo by binding: {}", options.foo);
    println!("karoucho by binding: {}", options.karoucho);
}
```

When using non-strings as keys, `serde` appends type information to the key name. The suffix must be then given when querying the value:


```rust
use config::{*, ext::*};

fn main() {
    let mut value: HashMap<i32, i32> = HashMap::new();
    value.insert(-32, 56);
    let config = DefaultConfigurationBuilder::new()
        .add_struct(value)
        .build()
        .unwrap();
    let loaded = config.get("-32_i32");
    assert_eq!(loaded.unwrap().as_str(), "56");
}

```

Also, note that binding is then not possible, since a data structure member name cannot start with a digit or a sign.
`bool` keys are not added any type suffix to the generated key, and can be used to bind into a data structure.
However, this can be applied in a limited number of scenarios and using non-string as keys is generally not supported.


