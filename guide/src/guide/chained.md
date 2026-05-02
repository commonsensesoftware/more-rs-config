# Chained Configuration Provider

>These features are only available if the **chained** feature is activated

Although it is not a very common usage scenario, you may encounter a scenario where you need to _chain_ multiple
[configurations](abstractions.md#configuration) from different sources into a unified
[configuration](abstractions.md#configuration). A practical example would be distinct
[configurations](abstractions.md#configuration) defined by different crates.

Let's assume that `crate1` defines its default configuration as:

```rust
use config::{Configuration, Result, prelude::*};

fn default_config() -> Result<Configuration> {
    config::builder()
        .add_in_memory(&[("Mem1:KeyInMem1", "ValueInMem1")])
        .add_in_memory(&[("Mem2:KeyInMem2", "ValueInMem2")])
        .build()
}
```

Let's assume that `crate2` defines its default configuration as:

```rust
use config::{Configuration, Result, prelude::*};

fn default_config() -> Result<Configuration> {
    config::builder()
        .add_in_memory(&[("Mem3:KeyInMem3", "ValueInMem3")])
        .build()
}
```

An application can now compose the `crate1` and `crate2` configurations into its own configuration.

```rust
use config::prelude::*;
use crate1;
use crate2;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error + 'static>> {
    let config = config::builder()
        .add_configuration(crate1::default_config()?)
        .add_configuration(crate2::default_config()?)
        .add_env_vars()
        .add_command_line()
        .build()?;

    println!("Mem1:KeyInMem1 = {}", config.get("mem1:keyinmem1").unwrap());
    println!("Mem2:KeyInMem2 = {}", config.get("Mem2:KeyInMem2").unwrap());
    println!("Mem3:KeyInMem3 = {}", config.get("MEM3:KEYINMEM3").unwrap());

    Ok(())
}
```

This configuration would output:

```text
Mem1:KeyInMem1 = ValueInMem1
Mem2:KeyInMem2 = ValueInMem2
Mem3:KeyInMem3 = ValueInMem3
```