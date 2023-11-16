# Command-Line Configuration Provider

>These features are only available if the **cmd** feature is activated

The `CommandLineConfigurationProvider` loads configuration from command-line argument key-value pairs. Configuration values set on the command-line can be used to override configuration values set with all the other configuration providers. When used, it is recommended that this is the last configuration provider added.

## Command-line Arguments

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

Within the same command, don't mix command-line argument key-value pairs that use `=` with key-value pairs that use a space.

```rust
use config::{*, ext::*};

fn main() {
    let config = DefaultConfigurationBuilder::new()
        .add_command_line()
        .build()
        .unwrap();

    println!("Name = {}", config.section("Position").get("Name").unwrap());
}

```

## Switch Mappings

Switch mappings allow key name replacement logic. Provide a hash map of switch replacements to the `add_command_line_map` method.

When the switch mappings hash map is used, the hash map is checked for a key that matches the key provided by a command-line argument. If the command-line key is found in the hash map, the hash map value is passed back to set the key-value pair into the application's configuration. A
switch mapping is required for any command-line key prefixed with a single dash (`-`).

Switch mappings hash map key rules:

- Switches must start with `-` or `--`.
- The switch mappings hash map must not contain duplicate keys.

To use a switch mappings hash map, pass it into the call to `add_command_line_map`:

```rust
use config::{*, ext::*};

fn main() {
    let switch_mappings = [
        ("-k1", "key1"),
        ("-k2", "key2"),
        ("--alt3", "key3"),
        ("--alt4", "key4"),
        ("--alt5", "key5"),
        ("--alt6", "key6"),
    ];
    let config = DefaultConfigurationBuilder::new()
        .add_command_line_map(&switch_mappings)
        .build()
        .unwrap();

    for (key, value) in config.iter() {
        println!("{} = {}", key, value);
    }
}
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

## Filtering

By default, the sequence of arguments provided by `std::env::args()` is supplied to the `CommandLineConfigurationSource`. If custom filtering is required, no extension method is provided to do so. The setup is still trivial albeit more verbose.

```rust
fn main() {
    let mut args: Vec<_> = std::env::args().collect();

    // TODO: apply filtering

    let cmd = CommandLineConfigurationSource::from(args.iter());
    let mut builder = DefaultConfigurationBuilder::new();

    builder.add(Box::new(cmd));

    let config = builder.build().unwrap();

    for (key, value) in config.iter() {
        println!("{} = {}", key, value);
    }
}
```