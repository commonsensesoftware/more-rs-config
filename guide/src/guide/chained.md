# Chained Configuration Provider

>These features are only available if the **chained** feature is activated

Although it is not a very common usage scenario, you may encounter a scenario where you need to _chain_ multiple [configurations](abstractions.md#configuration) from different sources into a unified [configuration](abstractions.md#configuration). A practical example would be distinct [configurations](abstractions.md#configuration) defined by different crates.

Let's assume that `crate1` defines its default configuration as:

```rust
use config::{*, ext::*};

fn default_config() -> Box<dyn ConfigurationRoot> {
    DefaultConfigurationBuilder::new()
        .add_in_memory(&[("Mem1:KeyInMem1", "ValueInMem1")])
        .add_in_memory(&[("Mem2:KeyInMem2", "ValueInMem2")])
        .build()
        .unwrap()
}
```

Let's assume that `crate2` defines its default configuration as:

```rust
use config::{*, ext::*};

fn default_config() -> Box<dyn ConfigurationRoot> {
    DefaultConfigurationBuilder::new()
        .add_in_memory(&[("Mem3:KeyInMem3", "ValueInMem3")])
        .build()
        .unwrap()
}
```

An application can now compose the `crate1` and `crate2` configurations into its own configuration.

```rust
use config::{*, ext::*};
use crate1;
use crate2;

fn main() {
    let root = DefaultConfigurationBuilder::new()
        .add_configuration(crate1::default_config().as_config())
        .add_configuration(crate2::default_config().as_config())
        .add_env_vars()
        .add_command_line()
        .build()
        .unwrap();

    println!("mem1:keyinmem1 = {}", root.get("mem1:keyinmem1").unwrap());
    println!("Mem2:KeyInMem2 = {}", root.get("Mem2:KeyInMem2").unwrap());
    println!("MEM3:KEYINMEM3 = {}", root.get("ValueInMem3").unwrap());
}
```

This configuration would output:

```text
mem1:keyinmem1 = mem1:keyinmem1
Mem2:KeyInMem2 = Mem2:KeyInMem2
MEM3:KEYINMEM3 = ValueInMem3
```