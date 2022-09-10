use crate::{
    util::accumulate_child_keys, ConfigurationBuilder, ConfigurationProvider, ConfigurationSource,
};
use std::borrow::Cow;
use std::collections::HashMap;

fn to_pascal_case<T: AsRef<str>>(text: T) -> String {
    let parts = text.as_ref().split('-');
    let mut pascal_case = String::with_capacity(text.as_ref().len());

    for part in parts {
        let mut chars = part.chars();

        if let Some(first) = chars.next() {
            pascal_case.push(first.to_ascii_uppercase());

            for ch in chars {
                pascal_case.push(ch);
            }
        }
    }

    pascal_case
}

/// Represents a [configuration provider](trait.ConfigurationProvider.html) that
/// provides command line configuration values.
pub struct CommandLineConfigurationProvider {
    data: HashMap<String, (String, String)>,
    args: Vec<String>,
    switch_mappings: HashMap<String, String>,
}

impl CommandLineConfigurationProvider {
    /// Initializes a new command line configuration provider.
    ///
    /// # Arguments
    ///
    /// * `args` - The command line arguments
    /// * `switch_mappings` - The mapping of switches to configuration values
    ///
    /// # Remarks
    ///
    /// Only switch mapping keys that start with `--` or `-` are acceptable. Command
    /// line arguments may start with `--`, `-`, or `/`
    pub fn new(args: Vec<String>, switch_mappings: HashMap<String, String>) -> Self {
        Self {
            data: Default::default(),
            args,
            switch_mappings: switch_mappings
                .iter()
                .filter(|m| m.0.starts_with("--") || m.0.starts_with('-'))
                .map(|(k, v)| (k.to_uppercase(), v.clone()))
                .collect(),
        }
    }
}

impl ConfigurationProvider for CommandLineConfigurationProvider {
    fn get(&self, key: &str) -> Option<&str> {
        self.data.get(&key.to_uppercase()).map(|t| t.1.as_str())
    }

    fn load(&mut self) {
        let mut data = HashMap::new();
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
            let mut key: String;
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
                    current
                        .chars()
                        .skip(start)
                        .take(separator - start)
                        .collect()
                };

                value = current.chars().skip(separator + 1).collect();
            } else {
                if start == 0 {
                    continue;
                }

                key = if let Some(mapping) = self.switch_mappings.get(&current.to_uppercase()) {
                    mapping.clone()
                } else if start == 0 {
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

            key = to_pascal_case(key);
            data.insert(key.to_uppercase(), (key, value));
        }

        self.data = data;
    }

    fn child_keys(&self, earlier_keys: &mut Vec<String>, parent_path: Option<&str>) {
        accumulate_child_keys(&self.data, earlier_keys, parent_path)
    }
}

/// Represents a [configuration source](trait.ConfigurationSource.html) for command line data.
#[derive(Default)]
pub struct CommandLineConfigurationSource {
    /// Gets or sets a collection of key/value pairs representing the mapping between
    /// switches and configuration keys.
    pub switch_mappings: HashMap<String, String>,

    /// Gets or sets the command line arguments.
    pub args: Vec<String>,
}

impl CommandLineConfigurationSource {
    /// Initializes a new command line configuration source.
    ///
    /// # Arguments
    ///
    /// * `args` - The command line arguments
    /// * `switch_mappings` - The mapping of switches to configuration values
    pub fn new(args: Vec<String>, switch_mappings: HashMap<String, String>) -> Self {
        Self {
            args,
            switch_mappings,
        }
    }
}

impl ConfigurationSource for CommandLineConfigurationSource {
    fn build(&self, _builder: &dyn ConfigurationBuilder) -> Box<dyn ConfigurationProvider> {
        Box::new(CommandLineConfigurationProvider::new(
            self.args.clone(),
            self.switch_mappings.clone(),
        ))
    }
}

pub mod ext {

    use super::*;

    /// Defines extension methods for the [ConfigurationBuilder](trait.ConfigurationBuilder.html) trait.
    pub trait CommandLineConfigurationBuilderExtensions {
        /// Adds the command line configuration source using the specified data.
        ///
        /// # Arguments
        ///
        /// * `args` - The command line arguments
        fn add_command_line(&mut self, args: Vec<String>) -> &mut Self;

        /// Adds the command line configuration source using the specified data.
        ///
        /// # Arguments
        ///
        /// * `args` - The command line arguments
        /// * `switch_mappings` - The mapping of switches to configuration values
        fn add_command_line_map(
            &mut self,
            args: Vec<String>,
            switch_mappings: HashMap<String, String>,
        ) -> &mut Self;
    }

    impl CommandLineConfigurationBuilderExtensions for dyn ConfigurationBuilder {
        fn add_command_line(&mut self, args: Vec<String>) -> &mut Self {
            self.add(Box::new(CommandLineConfigurationSource::new(
                args,
                Default::default(),
            )));
            self
        }

        fn add_command_line_map(
            &mut self,
            args: Vec<String>,
            switch_mappings: HashMap<String, String>,
        ) -> &mut Self {
            self.add(Box::new(CommandLineConfigurationSource::new(
                args,
                switch_mappings,
            )));
            self
        }
    }

    impl<T: ConfigurationBuilder> CommandLineConfigurationBuilderExtensions for T {
        fn add_command_line(&mut self, args: Vec<String>) -> &mut Self {
            self.add(Box::new(CommandLineConfigurationSource::new(
                args,
                Default::default(),
            )));
            self
        }

        fn add_command_line_map(
            &mut self,
            args: Vec<String>,
            switch_mappings: HashMap<String, String>,
        ) -> &mut Self {
            self.add(Box::new(CommandLineConfigurationSource::new(
                args,
                switch_mappings,
            )));
            self
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn load_should_ignore_unknown_arguments() {
        // arrange
        let args: Vec<_> = vec!["foo", "/bar=baz"]
            .into_iter()
            .map(String::from)
            .collect();
        let mut provider = CommandLineConfigurationProvider::new(args, Default::default());
        let mut child_keys = Vec::with_capacity(2);

        // act
        provider.load();
        provider.child_keys(&mut child_keys, None);

        // assert
        assert_eq!(child_keys.len(), 1);
        assert_eq!(provider.get("bar").unwrap(), "baz");
    }

    #[test]
    fn load_should_ignore_arguments_in_the_middle() {
        // arrange
        let args: Vec<_> = vec![
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
        .into_iter()
        .map(String::from)
        .collect();
        let mut provider = CommandLineConfigurationProvider::new(args, Default::default());
        let mut child_keys = Vec::with_capacity(5);

        // act
        provider.load();
        provider.child_keys(&mut child_keys, None);

        // assert
        // assert_eq!(child_keys.len(), 5);
        assert_eq!(provider.get("Key1").unwrap(), "Value1");
        assert_eq!(provider.get("Key2").unwrap(), "Value2");
        assert_eq!(provider.get("Key3").unwrap(), "Value3");
        assert_eq!(provider.get("Key4").unwrap(), "Value4");
        assert_eq!(provider.get("Key5").unwrap(), "Value5");
    }

    #[test]
    fn load_should_process_key_value_pairs_without_mappings() {
        // arrange
        let args: Vec<_> = vec![
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
        .into_iter()
        .map(String::from)
        .collect();
        let mut provider = CommandLineConfigurationProvider::new(args, Default::default());

        // act
        provider.load();

        // assert
        assert_eq!(provider.get("Key1").unwrap(), "Value1");
        assert_eq!(provider.get("Key2").unwrap(), "Value2");
        assert_eq!(provider.get("Key3").unwrap(), "Value3");
        assert_eq!(provider.get("Key4").unwrap(), "Value4");
        assert_eq!(provider.get("Key5").unwrap(), "Value5");
        assert_eq!(provider.get("Single").unwrap(), "1");
        assert_eq!(provider.get("TwoPart").unwrap(), "2");
    }

    #[test]
    fn load_should_process_key_value_pairs_with_mappings() {
        // arrange
        let args: Vec<_> = vec![
            "-K1=Value1",
            "--Key2=Value2",
            "/Key3=Value3",
            "--Key4",
            "Value4",
            "/Key5",
            "Value5",
            "/Key6=Value6",
        ]
        .into_iter()
        .map(String::from)
        .collect();
        let switch_mappings: HashMap<_, _> = vec![
            ("-K1", "LongKey1"),
            ("--Key2", "SuperLongKey2"),
            ("--Key6", "SuchALongKey6"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v.to_owned()))
        .collect();
        let mut provider = CommandLineConfigurationProvider::new(args, switch_mappings);

        // act
        provider.load();

        // assert
        assert_eq!(provider.get("LongKey1").unwrap(), "Value1");
        assert_eq!(provider.get("SuperLongKey2").unwrap(), "Value2");
        assert_eq!(provider.get("Key3").unwrap(), "Value3");
        assert_eq!(provider.get("Key4").unwrap(), "Value4");
        assert_eq!(provider.get("Key5").unwrap(), "Value5");
        assert_eq!(provider.get("SuchALongKey6").unwrap(), "Value6");
    }

    #[test]
    fn load_should_override_value_when_key_is_duplicated() {
        // arrange
        let args: Vec<_> = vec!["/Key1=Value1", "--Key1=Value2"]
            .into_iter()
            .map(String::from)
            .collect();
        let mut provider = CommandLineConfigurationProvider::new(args, Default::default());

        // act
        provider.load();

        // assert
        assert_eq!(provider.get("Key1").unwrap(), "Value2");
    }

    #[test]
    fn load_should_ignore_key_when_value_is_missing() {
        // arrange
        let args: Vec<_> = vec!["--Key1", "Value1", "/Key2"]
            .into_iter()
            .map(String::from)
            .collect();
        let mut provider = CommandLineConfigurationProvider::new(args, Default::default());
        let mut child_keys = Vec::with_capacity(2);

        // act
        provider.load();
        provider.child_keys(&mut child_keys, None);

        // assert
        assert_eq!(child_keys.len(), 1);
        assert_eq!(provider.get("Key1").unwrap(), "Value1");
    }

    #[test]
    fn load_should_ignore_unrecognizable_argument() {
        // arrange
        let args: Vec<_> = vec!["ArgWithoutPrefixAndEqualSign"]
            .into_iter()
            .map(String::from)
            .collect();
        let mut provider = CommandLineConfigurationProvider::new(args, Default::default());
        let mut child_keys = Vec::with_capacity(1);

        // act
        provider.load();
        provider.child_keys(&mut child_keys, None);

        // assert
        assert!(child_keys.is_empty());
    }

    #[test]
    fn load_should_ignore_argument_when_short_switch_is_undefined() {
        // arrange
        let args: Vec<_> = vec!["-Key1", "Value1"]
            .into_iter()
            .map(String::from)
            .collect();
        let switch_mappings: HashMap<_, _> = vec![("-Key2", "LongKey2")]
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect();
        let mut provider = CommandLineConfigurationProvider::new(args, switch_mappings);
        let mut child_keys = Vec::with_capacity(1);

        // act
        provider.load();
        provider.child_keys(&mut child_keys, Some(""));

        // assert
        assert!(child_keys.is_empty());
    }

    #[test]
    fn to_pascal_case_should_normalize_argument_name() {
        // arrange
        let argument = "no-build";
    
        // act
        let pascal_case = to_pascal_case(argument);
    
        // assert
        assert_eq!(pascal_case, "NoBuild");
    }
}
