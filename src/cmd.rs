use crate::{
    util::*, ConfigurationBuilder, ConfigurationProvider, ConfigurationSource, LoadResult, Value,
};
use std::borrow::Cow;
use std::collections::HashMap;

/// Represents a [`ConfigurationProvider`](crate::ConfigurationProvider) that
/// provides command line configuration values.
pub struct CommandLineConfigurationProvider {
    data: HashMap<String, (String, Value)>,
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
            switch_mappings,
        }
    }
}

impl ConfigurationProvider for CommandLineConfigurationProvider {
    fn get(&self, key: &str) -> Option<Value> {
        self.data.get(&key.to_uppercase()).map(|t| t.1.clone())
    }

    fn load(&mut self) -> LoadResult {
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

            key = to_pascal_case_parts(key, '-');
            data.insert(key.to_uppercase(), (key, value.into()));
        }

        data.shrink_to_fit();
        self.data = data;
        Ok(())
    }

    fn child_keys(&self, earlier_keys: &mut Vec<String>, parent_path: Option<&str>) {
        accumulate_child_keys(&self.data, earlier_keys, parent_path)
    }
}

/// Represents a [`ConfigurationSource`](crate::ConfigurationSource) for command line data.
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
    ///
    /// # Remarks
    ///
    /// Only switch mapping keys that start with `--` or `-` are acceptable. Command
    /// line arguments may start with `--`, `-`, or `/`.
    pub fn new<I, S1, S2>(args: I, switch_mappings: &[(S2, S2)]) -> Self
    where
        I: Iterator<Item = S1>,
        S1: AsRef<str>,
        S2: AsRef<str>,
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

impl<I, S> From<I> for CommandLineConfigurationSource
where
    I: Iterator<Item = S>,
    S: AsRef<str>,
{
    fn from(value: I) -> Self {
        let switch_mappings = Vec::<(&str, &str)>::with_capacity(0);
        Self::new(value, &switch_mappings)
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

    /// Defines extension methods for [`ConfigurationBuilder`](crate::ConfigurationBuilder).
    pub trait CommandLineConfigurationBuilderExtensions {
        /// Adds the command line configuration source.
        fn add_command_line(&mut self) -> &mut Self;

        /// Adds the command line configuration source.
        ///
        /// # Arguments
        ///
        /// * `switch_mappings` - The mapping of switches to configuration values
        fn add_command_line_map<S: AsRef<str>>(&mut self, switch_mappings: &[(S, S)]) -> &mut Self;
    }

    impl CommandLineConfigurationBuilderExtensions for dyn ConfigurationBuilder + '_ {
        fn add_command_line(&mut self) -> &mut Self {
            self.add(Box::new(CommandLineConfigurationSource::from(
                std::env::args(),
            )));
            self
        }

        fn add_command_line_map<S: AsRef<str>>(&mut self, switch_mappings: &[(S, S)]) -> &mut Self {
            self.add(Box::new(CommandLineConfigurationSource::new(
                std::env::args(),
                switch_mappings,
            )));
            self
        }
    }

    impl<T: ConfigurationBuilder> CommandLineConfigurationBuilderExtensions for T {
        fn add_command_line(&mut self) -> &mut Self {
            self.add(Box::new(CommandLineConfigurationSource::from(
                std::env::args(),
            )));
            self
        }

        fn add_command_line_map<S: AsRef<str>>(&mut self, switch_mappings: &[(S, S)]) -> &mut Self {
            self.add(Box::new(CommandLineConfigurationSource::new(
                std::env::args(),
                switch_mappings,
            )));
            self
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    struct TestConfigurationBuilder;

    impl ConfigurationBuilder for TestConfigurationBuilder {
        fn properties(&self) -> &HashMap<String, Box<dyn std::any::Any>> {
            unimplemented!()
        }

        fn sources(&self) -> &[Box<dyn ConfigurationSource>] {
            unimplemented!()
        }

        fn add(&mut self, _source: Box<dyn ConfigurationSource>) {
            unimplemented!()
        }

        fn build(&self) -> Result<Box<dyn crate::ConfigurationRoot>, crate::ReloadError> {
            unimplemented!()
        }
    }

    #[test]
    fn load_should_ignore_unknown_arguments() {
        // arrange
        let args = ["foo", "/bar=baz"].iter();
        let source = CommandLineConfigurationSource::from(args);
        let mut provider = source.build(&TestConfigurationBuilder);
        let mut child_keys = Vec::with_capacity(2);

        // act
        provider.load().unwrap();
        provider.child_keys(&mut child_keys, None);

        // assert
        assert_eq!(child_keys.len(), 1);
        assert_eq!(provider.get("bar").unwrap().as_str(), "baz");
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
        let source = CommandLineConfigurationSource::from(args);
        let mut provider = source.build(&TestConfigurationBuilder);
        let mut child_keys = Vec::with_capacity(5);

        // act
        provider.load().unwrap();
        provider.child_keys(&mut child_keys, None);

        // assert
        assert_eq!(provider.get("Key1").unwrap().as_str(), "Value1");
        assert_eq!(provider.get("Key2").unwrap().as_str(), "Value2");
        assert_eq!(provider.get("Key3").unwrap().as_str(), "Value3");
        assert_eq!(provider.get("Key4").unwrap().as_str(), "Value4");
        assert_eq!(provider.get("Key5").unwrap().as_str(), "Value5");
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
        let source = CommandLineConfigurationSource::from(args);
        let mut provider = source.build(&TestConfigurationBuilder);

        // act
        provider.load().unwrap();

        // assert
        assert_eq!(provider.get("Key1").unwrap().as_str(), "Value1");
        assert_eq!(provider.get("Key2").unwrap().as_str(), "Value2");
        assert_eq!(provider.get("Key3").unwrap().as_str(), "Value3");
        assert_eq!(provider.get("Key4").unwrap().as_str(), "Value4");
        assert_eq!(provider.get("Key5").unwrap().as_str(), "Value5");
        assert_eq!(provider.get("Single").unwrap().as_str(), "1");
        assert_eq!(provider.get("TwoPart").unwrap().as_str(), "2");
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
        let source = CommandLineConfigurationSource::new(args, &switch_mappings);
        let mut provider = source.build(&TestConfigurationBuilder);

        // act
        provider.load().unwrap();

        // assert
        assert_eq!(provider.get("LongKey1").unwrap().as_str(), "Value1");
        assert_eq!(provider.get("SuperLongKey2").unwrap().as_str(), "Value2");
        assert_eq!(provider.get("Key3").unwrap().as_str(), "Value3");
        assert_eq!(provider.get("Key4").unwrap().as_str(), "Value4");
        assert_eq!(provider.get("Key5").unwrap().as_str(), "Value5");
        assert_eq!(provider.get("SuchALongKey6").unwrap().as_str(), "Value6");
    }

    #[test]
    fn load_should_override_value_when_key_is_duplicated() {
        // arrange
        let args = ["/Key1=Value1", "--Key1=Value2"].iter();
        let source = CommandLineConfigurationSource::from(args);
        let mut provider = source.build(&TestConfigurationBuilder);

        // act
        provider.load().unwrap();

        // assert
        assert_eq!(provider.get("Key1").unwrap().as_str(), "Value2");
    }

    #[test]
    fn load_should_ignore_key_when_value_is_missing() {
        // arrange
        let args = ["--Key1", "Value1", "/Key2"].iter();
        let source = CommandLineConfigurationSource::from(args);
        let mut provider = source.build(&TestConfigurationBuilder);
        let mut child_keys = Vec::with_capacity(2);

        // act
        provider.load().unwrap();
        provider.child_keys(&mut child_keys, None);

        // assert
        assert_eq!(child_keys.len(), 1);
        assert_eq!(provider.get("Key1").unwrap().as_str(), "Value1");
    }

    #[test]
    fn load_should_ignore_unrecognizable_argument() {
        // arrange
        let args = ["ArgWithoutPrefixAndEqualSign"].iter();
        let source = CommandLineConfigurationSource::from(args);
        let mut provider = source.build(&TestConfigurationBuilder);
        let mut child_keys = Vec::with_capacity(1);

        // act
        provider.load().unwrap();
        provider.child_keys(&mut child_keys, None);

        // assert
        assert!(child_keys.is_empty());
    }

    #[test]
    fn load_should_ignore_argument_when_short_switch_is_undefined() {
        // arrange
        let args = ["-Key1", "Value1"].iter();
        let switch_mappings = [("-Key2", "LongKey2")];
        let source = CommandLineConfigurationSource::new(args, &switch_mappings);
        let mut provider = source.build(&TestConfigurationBuilder);
        let mut child_keys = Vec::with_capacity(1);

        // act
        provider.load().unwrap();
        provider.child_keys(&mut child_keys, Some(""));

        // assert
        assert!(child_keys.is_empty());
    }
}
