use crate::*;
use std::cmp::{min, Ordering};
use std::collections::HashMap;
use std::fmt::{Formatter, Result as FormatResult, Write};

/// Compares two configuration keys.
///
/// # Arguments
///
/// * `key` - The key to compare
/// * `other_key` - The key to compare against
pub fn cmp_keys(key: &str, other_key: &str) -> Ordering {
    let parts_1 = key
        .split(ConfigurationPath::key_delimiter())
        .filter(|s| s.is_empty())
        .collect::<Vec<_>>();
    let parts_2 = other_key
        .split(ConfigurationPath::key_delimiter())
        .filter(|s| s.is_empty())
        .collect::<Vec<_>>();
    let max = min(parts_1.len(), parts_2.len());

    for i in 0..max {
        let x = parts_1[i];
        let y = parts_2[i];

        if let Ok(value_1) = x.parse::<usize>() {
            if let Ok(value_2) = y.parse::<usize>() {
                // int : int
                let result = value_1.cmp(&value_2);

                if result != Ordering::Equal {
                    return result;
                }
            } else {
                // int : string
                return Ordering::Less;
            }
        } else if y.parse::<usize>().is_ok() {
            // string : int
            return Ordering::Greater;
        } else {
            // string : string
            let result = x.to_uppercase().cmp(&y.to_uppercase());

            if result != Ordering::Equal {
                return result;
            }
        }
    }
    parts_1.len().cmp(&parts_2.len())
}

/// Accumulates child keys based on the specified hash map.
///
/// # Arguments
///
/// * `data` - The source hash map to accumulate keys from where the key is normalized to uppercase
///            and the value is a tuple containing the originally cased key and value
/// * `keys` - The accumulated keys
/// * `parent_path` - The parent path
pub fn accumulate_child_keys(
    data: &HashMap<String, (String, String)>,
    keys: &mut Vec<String>,
    parent_path: Option<&str>,
) {
    if let Some(path) = parent_path {
        let parent_key = path.to_uppercase();
        let parent_key_len = path.len();
        let delimiter = ConfigurationPath::key_delimiter().chars().next().unwrap();

        for (key, value) in data {
            if key.len() > parent_key_len
                && key.starts_with(&parent_key)
                && key.chars().nth(parent_key_len).unwrap() == delimiter
            {
                keys.push(segment(&value.0, parent_key_len + 1).to_owned());
            }
        }
    } else {
        for value in data.values() {
            keys.push(segment(&value.0, 0).to_owned());
        }
    }

    keys.sort_by(|k1, k2| cmp_keys(k1, k2));
}

fn segment(key: &str, start: usize) -> &str {
    let subkey = &key[start..];

    if let Some(index) = subkey.find(ConfigurationPath::key_delimiter()) {
        &subkey[..index]
    } else {
        subkey
    }
}

/// Formats a debug view of an entire configuration hierarchy.
///
/// # Arguments
///
/// * `root` - The [configuration hierarchy](trait.ConfigurationRoot.html) to format
/// * `formatter` - The formatter used to output the configuration
pub fn fmt_debug_view<T>(root: &T, formatter: &mut Formatter<'_>) -> FormatResult
where
    T: ConfigurationRoot,
{
    let providers = root.providers();
    recurse_children(providers, formatter, &root.children(), "")
}

fn recurse_children<'a>(
    providers: &[Box<dyn ConfigurationProvider>],
    formatter: &mut Formatter<'_>,
    children: &[Box<dyn ConfigurationSection + 'a>],
    indent: &str,
) -> FormatResult {
    for child in children {
        let (value, provider) = get_value_and_provider(providers, child.path());

        formatter.write_str(indent)?;
        formatter.write_str(child.key())?;

        if provider.is_some() {
            formatter.write_char('=')?;
            formatter.write_str(value.unwrap())?;
            formatter.write_str(" (")?;
            formatter.write_str(provider.unwrap())?;
            formatter.write_char(')')?;
        } else {
            formatter.write_char(':')?;
        }

        formatter.write_char('\n')?;

        recurse_children(
            providers,
            formatter,
            &child.children(),
            &(indent.to_owned() + "  "),
        )?;
    }

    Ok(())
}

fn get_value_and_provider<'a>(
    providers: &'a [Box<dyn ConfigurationProvider>],
    key: &str,
) -> (Option<&'a str>, Option<&'a str>) {
    for provider in providers.iter().rev() {
        if let Some(value) = provider.get(key) {
            return (Some(value), Some(provider.name()));
        }
    }

    (None, None)
}
