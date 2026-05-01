use crate::FileSourceBuilder;
use std::path::Path;

/// Provides extension methods to create a [files source builder](FileSourceBuilder).
pub trait FileSourceBuilderExt {
    /// Creates a new [file source builder](FileSourceBuilder).
    fn is(&self) -> FileSourceBuilder;
}

impl<T: AsRef<Path>> FileSourceBuilderExt for T {
    fn is(&self) -> FileSourceBuilder {
        FileSourceBuilder::new(self.as_ref().to_path_buf())
    }
}
