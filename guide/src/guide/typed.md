{{#include links.md}}

# Typed Configuration Provider

>These features are only available if the **typed** feature is activated

The [typed::Provider] uses a typed data structure as the source input for configuration values. This is most useful as
an alternative default configuration to in-memory configuration, which uses loosely defined key/value pairs. This is
also useful when providing test values.

The following code adds a data structure to the configuration system and displays the settings:

```rust
use config::prelude::*;
use serde::{Deserialize, Serialize};
use std::error::Error;

#[derive(Clone, Default, Deserialize, Serialize)]
struct PerfSettings {
    cores: u8,
} 

#[derive(Clone, Default, Deserialize, Serialize)]
struct AppOptions {
    title: String,
    perf: PerfSettings,
}

fn main() -> Result<(), Box<dyn Error + 'static>> {
    let default = AppOptions {
        title: String::from("Banana processor"),
        perf: SubOptions{ cores: 7 },
    };
    let config = config::builder().add_typed(default).build()?;
    let title = config.get("Title").unwrap();
    let cores = config.get("Perf:Cores").unwrap();

    println!("Title: {title}\n\ 
              Cores: {cores}");

    Ok(())
}
```

Although bare primitives and collections of primitives are serializable, they are not supported because there is no
corresponding key which would otherwise be derived from an associated complex structure. This limitation only applies
to the root value. Nested data structures can be primitives, tuples, and so on. The one exception is a typed
`HashMap<String, _>` because keys are present; however, this is only slightly better than using the
[in-memory provider](memory.md).
