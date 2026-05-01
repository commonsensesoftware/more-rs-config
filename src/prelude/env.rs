use crate::{env, Builder};

/// Defines environment variable extension methods for a [configuration builder](Builder).
pub trait EnvVarsExt: Sized {
    /// Adds environment variables as a configuration source.
    fn add_env_vars(self) -> Self {
        self.add_env_vars_with_prefix("")
    }

    /// Adds environment variables as a configuration source.
    ///
    /// # Arguments
    ///
    /// * `prefix` - The prefix that environment variable names must start with. The prefix will be removed from
    ///   the environment variable names.
    fn add_env_vars_with_prefix<S: Into<String>>(self, prefix: S) -> Self;
}
impl EnvVarsExt for Builder {
    fn add_env_vars_with_prefix<S: Into<String>>(mut self, prefix: S) -> Self {
        self.add(env::Source::new(prefix));
        self
    }
}
