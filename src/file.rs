use std::path::{Path, PathBuf};
use std::time::Duration;

/// Represents a file configuration source.
#[derive(Clone)]
pub struct FileSource {
    /// Gets or sets the source file path.
    pub path: PathBuf,

    /// Gets or sets a value indicating whether the file is optional.
    /// The default value is false.
    pub optional: bool,

    /// Gets or sets a value indicating whether the file will be loaded
    /// if the underlying file changes. The default value is false.
    pub reload_on_change: bool,

    /// Get or sets the amount of time to wait after a change before reloading.
    /// The default value is 250ms.
    ///
    /// # Remarks
    ///
    /// This helps avoid triggering reload before a file is completely written.
    pub reload_delay: Duration,
}

impl FileSource {
    /// Initializes a new file configuration source.
    ///
    /// # Arguments
    ///
    /// * `path` - The source file path
    /// * `optional` - Indicates whether the source file must exist
    /// * `reload_on_change` - Indicates if a reload should occur if the source file changes
    /// * `reload_delay` - The amount of delay before reload after the source file changes
    pub fn new(
        path: PathBuf,
        optional: bool,
        reload_on_change: bool,
        reload_delay: Option<Duration>,
    ) -> Self {
        Self {
            path,
            optional,
            reload_on_change,
            reload_delay: reload_delay.unwrap_or(Duration::from_millis(250)),
        }
    }

    /// Initializes a new, optional file configuration source.
    ///
    /// # Arguments
    ///
    /// * `path` - The source file path
    pub fn optional<P: AsRef<Path>>(path: P) -> Self {
        Self::new(path.as_ref().to_path_buf(), true, false, None)
    }
}

impl From<PathBuf> for FileSource {
    fn from(value: PathBuf) -> Self {
        Self::new(value, false, false, None)
    }
}

impl From<&PathBuf> for FileSource {
    fn from(value: &PathBuf) -> Self {
        Self::from(value.clone())
    }
}

impl From<&Path> for FileSource {
    fn from(value: &Path) -> Self {
        Self::from(value.to_path_buf())
    }
}

impl From<&str> for FileSource {
    fn from(value: &str) -> Self {
        Self::from(PathBuf::from(value))
    }
}

impl From<String> for FileSource {
    fn from(value: String) -> Self {
        Self::from(PathBuf::from(value))
    }
}

impl From<&String> for FileSource {
    fn from(value: &String) -> Self {
        Self::from(PathBuf::from(value))
    }
}

/// Represents a builder for a file source.
pub struct FileSourceBuilder {
    path: PathBuf,
    optional: bool,
    reload_on_change: bool,
    reload_delay: Option<Duration>,
}

impl FileSourceBuilder {
    /// Initializes a new file source builder.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to build a file source for
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            optional: false,
            reload_on_change: false,
            reload_delay: None,
        }
    }

    /// Indicates the file source is optional.
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    /// Indicates the file source can be reloaded.
    pub fn reloadable(mut self) -> Self {
        self.reload_on_change = true;
        self
    }

    /// Sets the delay to wait before reloading when a file source changes.
    pub fn reload_delay(mut self, delay: Duration) -> Self {
        self.reload_delay = Some(delay);
        self
    }

    /// Creates and returns a new [`FileSource`].
    pub fn build(&self) -> FileSource {
        FileSource::new(
            self.path.clone(),
            self.optional,
            self.reload_on_change,
            self.reload_delay,
        )
    }
}

impl From<FileSourceBuilder> for FileSource {
    fn from(value: FileSourceBuilder) -> Self {
        value.build()
    }
}

impl From<&FileSourceBuilder> for FileSource {
    fn from(value: &FileSourceBuilder) -> Self {
        value.build()
    }
}

pub mod ext {

    use super::*;

    /// Provides extension methods to create a [`FileSourceBuilder`].
    pub trait FileSourceBuilderExtensions {
        fn is(&self) -> FileSourceBuilder;
    }

    impl<T: AsRef<Path>> FileSourceBuilderExtensions for T {
        fn is(&self) -> FileSourceBuilder {
            FileSourceBuilder::new(self.as_ref().to_path_buf())
        }
    }
}
