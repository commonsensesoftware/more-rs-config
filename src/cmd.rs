use crate::{Properties, Result, Settings};
use std::{borrow::Cow, collections::HashMap, mem::take};

fn to_pascal_case(text: String, sep: char) -> String {
    let parts = text.split(sep);
    let mut pascal_case = String::with_capacity(text.len());

    for part in parts {
        let mut chars = part.chars();

        if let Some(first) = chars.next() {
            pascal_case.push_str(&first.to_uppercase().to_string());
            pascal_case.push_str(chars.as_str());
        }
    }

    pascal_case
}

#[derive(Debug)]
struct Provider {
    args: Vec<String>,
    switch_mappings: HashMap<String, String>,
}

impl Provider {
    #[inline]
    fn new(args: Vec<String>, switch_mappings: HashMap<String, String>) -> Self {
        Self { args, switch_mappings }
    }
}

impl crate::Provider for Provider {
    #[inline]
    fn name(&self) -> &str {
        "Command Line"
    }

    fn load(&self, settings: &mut Settings) -> Result {
        let mut args = self.args.iter();

        while let Some(arg) = args.next() {
            let mut current = Cow::Borrowed(arg);
            let start: usize = if arg.starts_with("--") {
                2
            } else if arg.starts_with('-') {
                1
            } else if arg.starts_with('/') {
                // "/SomeSwitch" is equivalent to "--SomeSwitch" when interpreting switch mappings
                let mut temp = arg.clone();
                temp.replace_range(0..1, "--");
                current = Cow::Owned(temp);
                2
            } else {
                0
            };
            let key: String;
            let value: String;

            if let Some(separator) = current.find('=') {
                let segment: String = current
                    .chars()
                    .take(separator)
                    .map(|c| c.to_ascii_uppercase())
                    .collect();

                key = if let Some(mapping) = self.switch_mappings.get(&segment) {
                    mapping.clone()
                } else if start == 1 {
                    continue;
                } else {
                    current.chars().skip(start).take(separator - start).collect()
                };

                value = current.chars().skip(separator + 1).collect();
            } else {
                if start == 0 {
                    continue;
                }

                key = if let Some(mapping) = self.switch_mappings.get(&current.to_uppercase()) {
                    mapping.clone()
                } else if start == 1 {
                    continue;
                } else {
                    current.chars().skip(start).collect()
                };

                if let Some(next) = args.next() {
                    value = next.clone();
                } else {
                    continue;
                }
            }

            settings.insert(to_pascal_case(key, '-'), value);
        }

        Ok(())
    }
}

/// Represents a [configuration source](Source) for command line data.
#[derive(Default)]
pub struct Source {
    /// Gets or sets the command line arguments.
    pub args: Vec<String>,

    /// Gets or sets a collection of key/value pairs representing the mapping between switches and configuration keys.
    pub switch_mappings: HashMap<String, String>,
}

impl Source {
    /// Initializes a new command line configuration source.
    ///
    /// # Arguments
    ///
    /// * `args` - The command line arguments
    /// * `switch_mappings` - The mapping of switches to configuration values
    ///
    /// # Remarks
    ///
    /// Only switch mapping keys that start with `--` or `-` are acceptable. Command line arguments may start with
    /// `--`, `-`, or `/`.
    pub fn new<I, V, S>(args: I, switch_mappings: &[(S, S)]) -> Self
    where
        I: Iterator<Item = V>,
        V: AsRef<str>,
        S: AsRef<str>,
    {
        Self {
            args: args.map(|a| a.as_ref().to_owned()).collect(),
            switch_mappings: switch_mappings
                .iter()
                .filter(|m| m.0.as_ref().starts_with("--") || m.0.as_ref().starts_with('-'))
                .map(|(k, v)| (k.as_ref().to_uppercase(), v.as_ref().to_owned()))
                .collect(),
        }
    }
}

impl<I, V> From<I> for Source
where
    I: Iterator<Item = V>,
    V: AsRef<str>,
{
    #[inline]
    fn from(value: I) -> Self {
        Self::new(value, &Vec::<(&str, &str)>::new())
    }
}

impl crate::Source for Source {
    fn build(&mut self, _properties: &mut Properties) -> Box<dyn crate::Provider> {
        Box::new(Provider::new(take(&mut self.args), take(&mut self.switch_mappings)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Properties, Source as _};

    #[test]
    fn to_pascal_case_should_normalize_argument_name() {
        // arrange
        let argument = "no-build";

        // act
        let pascal_case = to_pascal_case(argument.into(), '-');

        // assert
        assert_eq!(pascal_case, "NoBuild");
    }

    #[test]
    fn load_should_ignore_unknown_arguments() {
        // arrange
        let args = ["foo", "/bar=baz"].iter();
        let mut source = Source::from(args);
        let provider = source.build(&mut Properties::default());
        let mut settings = Settings::default();

        // act
        provider.load(&mut settings).unwrap();

        println!("{settings:?}");

        // assert
        assert_eq!(settings.len(), 1);
        assert_eq!(settings.get("bar"), Some("baz"));
    }

    #[test]
    fn load_should_ignore_arguments_in_the_middle() {
        // arrange
        let args = [
            "Key1=Value1",
            "--Key2=Value2",
            "/Key3=Value3",
            "Bogus1",
            "--Key4",
            "Value4",
            "Bogus2",
            "/Key5",
            "Value5",
            "Bogus3",
        ]
        .iter();
        let mut source = Source::from(args);
        let provider = source.build(&mut Properties::default());
        let mut settings = Settings::default();

        // act
        provider.load(&mut settings).unwrap();

        // assert
        assert_eq!(settings.get("Key1"), Some("Value1"));
        assert_eq!(settings.get("Key2"), Some("Value2"));
        assert_eq!(settings.get("Key3"), Some("Value3"));
        assert_eq!(settings.get("Key4"), Some("Value4"));
        assert_eq!(settings.get("Key5"), Some("Value5"));
    }

    #[test]
    fn load_should_process_key_value_pairs_without_mappings() {
        // arrange
        let args = [
            "Key1=Value1",
            "--Key2=Value2",
            "/Key3=Value3",
            "--Key4",
            "Value4",
            "/Key5",
            "Value5",
            "--single=1",
            "--two-part=2",
        ]
        .iter();
        let mut source = Source::from(args);
        let provider = source.build(&mut Properties::default());
        let mut settings = Settings::default();

        // act
        provider.load(&mut settings).unwrap();

        // assert
        assert_eq!(settings.get("Key1"), Some("Value1"));
        assert_eq!(settings.get("Key2"), Some("Value2"));
        assert_eq!(settings.get("Key3"), Some("Value3"));
        assert_eq!(settings.get("Key4"), Some("Value4"));
        assert_eq!(settings.get("Key5"), Some("Value5"));
        assert_eq!(settings.get("Single"), Some("1"));
        assert_eq!(settings.get("TwoPart"), Some("2"));
    }

    #[test]
    fn load_should_process_key_value_pairs_with_mappings() {
        // arrange
        let args = [
            "-K1=Value1",
            "--Key2=Value2",
            "/Key3=Value3",
            "--Key4",
            "Value4",
            "/Key5",
            "Value5",
            "/Key6=Value6",
        ]
        .iter();
        let switch_mappings = [
            ("-K1", "LongKey1"),
            ("--Key2", "SuperLongKey2"),
            ("--Key6", "SuchALongKey6"),
        ];
        let mut source = Source::new(args, &switch_mappings);
        let provider = source.build(&mut Properties::default());
        let mut settings = Settings::default();

        // act
        provider.load(&mut settings).unwrap();

        // assert
        assert_eq!(settings.get("LongKey1"), Some("Value1"));
        assert_eq!(settings.get("SuperLongKey2"), Some("Value2"));
        assert_eq!(settings.get("Key3"), Some("Value3"));
        assert_eq!(settings.get("Key4"), Some("Value4"));
        assert_eq!(settings.get("Key5"), Some("Value5"));
        assert_eq!(settings.get("SuchALongKey6"), Some("Value6"));
    }

    #[test]
    fn load_should_override_value_when_key_is_duplicated() {
        // arrange
        let args = ["/Key1=Value1", "--Key1=Value2"].iter();
        let mut source = Source::from(args);
        let provider = source.build(&mut Properties::default());
        let mut settings = Settings::default();

        // act
        provider.load(&mut settings).unwrap();

        // assert
        assert_eq!(settings.get("Key1"), Some("Value2"));
    }

    #[test]
    fn load_should_ignore_key_when_value_is_missing() {
        // arrange
        let args = ["--Key1", "Value1", "/Key2"].iter();
        let mut source = Source::from(args);
        let provider = source.build(&mut Properties::default());
        let mut settings = Settings::default();

        // act
        provider.load(&mut settings).unwrap();

        // assert
        assert_eq!(settings.len(), 1);
        assert_eq!(settings.get("Key1"), Some("Value1"));
    }

    #[test]
    fn load_should_ignore_unrecognizable_argument() {
        // arrange
        let args = ["ArgWithoutPrefixAndEqualSign"].iter();
        let mut source = Source::from(args);
        let provider = source.build(&mut Properties::default());
        let mut settings = Settings::default();

        // act
        provider.load(&mut settings).unwrap();

        // assert
        assert!(settings.is_empty());
    }

    #[test]
    fn load_should_ignore_argument_when_short_switch_is_undefined() {
        // arrange
        let args = ["-Key1", "Value1"].iter();
        let switch_mappings = [("-Key2", "LongKey2")];
        let mut source = Source::new(args, &switch_mappings);
        let provider = source.build(&mut Properties::default());
        let mut settings = Settings::default();

        // act
        provider.load(&mut settings).unwrap();

        // assert
        assert!(settings.is_empty());
    }
}
