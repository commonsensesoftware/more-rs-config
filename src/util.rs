use crate::*;
use std::cmp::{min, Ordering};
use std::collections::HashMap;
use std::fmt::{Formatter, Result as FormatResult, Write};

#[cfg(feature = "json")]
pub(crate) fn to_pascal_case<T: AsRef<str>>(text: T) -> String {
    let mut chars = text.as_ref().chars();

    if let Some(first) = chars.next() {
        first.to_uppercase().collect::<String>() + chars.as_str()
    } else {
        String::with_capacity(0)
    }
}

#[cfg(feature = "cmd")]
pub(crate) fn to_pascal_case_parts<T: AsRef<str>>(text: T, sep: char) -> String {
    let parts = text.as_ref().split(sep);
    let mut pascal_case = String::with_capacity(text.as_ref().len());

    for part in parts {
        let mut chars = part.chars();

        if let Some(first) = chars.next() {
            pascal_case.push_str(&first.to_uppercase().to_string());
            pascal_case.push_str(chars.as_str());
        }
    }

    pascal_case
}

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
    recurse_children(root, &root.children(), formatter, "")
}

fn recurse_children<T: ConfigurationRoot>(
    root: &T,
    children: &[Box<dyn ConfigurationSection>],
    formatter: &mut Formatter<'_>,
    indent: &str,
) -> FormatResult {
    for child in children {
        formatter.write_str(indent)?;
        formatter.write_str(child.key())?;

        let mut found = false;

        for provider in root.providers().rev() {
            if let Some(value) = provider.get(child.path()) {
            formatter.write_char('=')?;
            formatter.write_str(&value)?;
            formatter.write_str(" (")?;
                formatter.write_str(provider.name())?;
            formatter.write_char(')')?;
                found = true;
                break;
            }
        }

        if !found {
            formatter.write_char(':')?;
        }

        formatter.write_char('\n')?;

        recurse_children(
            root,
            &child.children(),
            formatter,
            &(indent.to_owned() + "  "),
        )?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn to_pascal_case_should_normalize_argument_name() {
        // arrange
        let argument = "noBuild";

        // act
        let pascal_case = to_pascal_case(argument);

        // assert
        assert_eq!(pascal_case, "NoBuild");
    }

    #[test]
    fn to_pascal_case_parts_should_normalize_argument_name() {
        // arrange
        let argument = "no-build";

        // act
        let pascal_case = to_pascal_case_parts(argument, '-');

        // assert
        assert_eq!(pascal_case, "NoBuild");
    }
}
