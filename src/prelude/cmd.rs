use crate::{cmd, Builder};
use std::env;

/// Defines command line extension methods for a [configuration builder](Builder).
pub trait CommandLineExt: Sized {
    /// Adds a command line configuration source.
    fn add_command_line(self) -> Self;

    /// Adds a command line configuration source.
    ///
    /// # Arguments
    ///
    /// * `switch_mappings` - The mapping of switches to configuration values
    fn add_command_line_map<S: AsRef<str>>(self, switch_mappings: &[(S, S)]) -> Self;
}

impl CommandLineExt for Builder {
    fn add_command_line(mut self) -> Self {
        self.add(cmd::Source::from(env::args()));
        self
    }

    fn add_command_line_map<S: AsRef<str>>(mut self, switch_mappings: &[(S, S)]) -> Self {
        self.add(cmd::Source::new(env::args(), switch_mappings));
        self
    }
}
