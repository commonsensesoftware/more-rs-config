{{#include links.md}}

# Troubleshooting

Multiple sources of configuration information can be difficult and confusing to track down why a particular configured
value is the value that it is. Several diagnostic capabilities are provided to help you trace how a value was configured.

Source tracing is supported for up to 8 configuration providers. The limit of 8 is a design choice to keep the cost of
tracing low. It's unlikely an application would have more than 8 configuration providers at a time.

## Tracing

The configuration system provides limited support for the [tracing](https://crates.io/crates/tracing) crate.

### Configuration Reload

Whenever a [configuration reloads][ReloadableConfiguration], it will log a `Level::TRACE` message indicating a reload has
occurred. If the reload fails, a `Level::ERROR` message will be recorded with the error details so that background
failures can be observed and triaged.

Whenever an existing configuration value is overridden by another configuration provider, a `Level::TRACE` message will
log the key, old value, new value, who provided the old value, and who override the key with a new value. This can
help identify unexpected configuration resolution in an application.

## Formatting

A [configuration][Configuration] can also be inspected by simply printing it out. The implementation of the `Display`
trait will output the configuration hierarchy [section][Section] by section. The implementation also supports an
alternate output form that includes the corresponding providers associated with a configured value. The default
alternate behavior expands all providers that set the configuration value. The number of providers that are output can
be controlled by the formatting width. The configuration providers that are output are ranked most recent to oldest. A
value of `0`, indicates the last effective provider for a configuration value.

### Default Output

The default output using `format!("{config}")` has the hierarchical format:

```text
Section1:
  Key1 = Value1
Section2:
  Key2 = Value2
```

### Show Providers

The alternate output using `format!("{config:#}")` has the format:

```text
Section1:
  Key1 = Value1 (Memory)
Section2:
  Key2 = Value2 (Memory → overrides.json)
```

### Show Effective Provider Only

The alternate and width output using `format!("{config:#1}")` has the format:

```text
Section1:
  Key1 = Value1 (Memory)
Section2:
  Key2 = Value2 (overrides.json)
```
