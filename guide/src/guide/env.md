# Environment Variable Configuration Provider

>These features are only available if the **env** feature is activated

The `EnvironmentVariablesConfigurationProvider` loads configuration from environment variable key-value pairs.

The `:` separator doesn't work with environment variable hierarchical keys on all platforms. `__`, the double underscore, is:

- Supported by all platforms; for example, the `:` separator is not supported by Bash, but `__` is.
- Automatically replaced by a `:`

```bash
export MyKey="My key from Environment"
export Position__Title=Console
export Position__Name="John Doe"
```

Call `add_env_vars` to add environment variables or `add_env_vars_with_prefix` with a string to specify a prefix for environment variables:

```rust
use config::{*, ext::*};

fn main() {
    let config = DefaultConfigurationBuilder::new()
            .add_env_vars_with_prefix("MyCustomPrefix_")
            .build()
            .unwrap();
    
    for (key, value) in config.iter() {
        println!("{} = {}", key, value);
    }
}
```

Environment variables set with the `MyCustomPrefix_` prefix override the default configuration providers. This includes environment variables without the prefix. The prefix is stripped off when the configuration key-value pairs are read.

```bash
export MyCustomPrefix_MyKey="My key with MyCustomPrefix_ Environment"
export MyCustomPrefix_Position__Title="Custom Editor"
export MyCustomPrefix_Position__Name="Jane Doe"
```

## Naming of Environment Variables

Environment variable names reflect the structure of an `appsettings.json` file. Each element in the hierarchy is separated by a double underscore. When the element structure includes an array, the array index should be treated as an additional element name in this path. Consider the following `appsettings.json` file and its equivalent values represented as environment variables.

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