const EMPTY: &str = "";
const KEY_DELIMITER: &str = ":";

/// Represents a configuration path.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ConfigurationPath {
    Absolute,
    Relative
}

impl ConfigurationPath {
    /// Gets the key delimiter used in configuration paths.
    pub fn key_delimiter() -> &'static str {
        KEY_DELIMITER
    }

    /// Combines the specified segments into one path.
    ///
    /// # Arguments
    ///
    /// * `segments` - The segments to combine into a path
    pub fn combine(segments: &[&str]) -> String {
        segments.join(KEY_DELIMITER)
    }

    /// Extracts the last path segment from the path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to extract the key from
    pub fn section_key(path: &str) -> &str {
        if let Some(index) = path.rfind(KEY_DELIMITER) {
            &path[(index + 1)..]
        } else {
            path
        }
    }

    /// Extracts the path corresponding to the parent node for a given path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path to extract the parent path from
    pub fn parent_path(path: &str) -> &str {
        if let Some(index) = path.rfind(KEY_DELIMITER) {
            &path[..index]
        } else {
            EMPTY
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use test_case::test_case;

    #[test_case(&["parent", ""], "parent:" ; "with 1 segment")]
    #[test_case(&["parent", "", ""], "parent::" ; "with 2 segments")]
    #[test_case(&["parent", "", "", "key"], "parent:::key" ; "with segments in between")]
    fn combine_with_empty_segment_leaves_delimiter(segments: &[&str], expected: &str) {
        // arrange

        // act
        let path = ConfigurationPath::combine(segments);

        // assert
        assert_eq!(&path, expected);
    }

    #[test_case("", "" ; "when empty")]
    #[test_case(":::", "" ; "when only delimiters")]
    #[test_case("a::b:::c", "c" ; "with empty segments in the middle")]
    #[test_case("a:::b:", "" ; "when last segment is empty")]
    #[test_case("key", "key" ; "with no parent")]
    #[test_case(":key", "key" ; "with 1 empty parent")]
    #[test_case("::key", "key" ; "with 2 empty parents")]
    #[test_case("parent:key", "key" ; "with parent")]
    fn section_key_should_return_expected_segment(path: &str, expected: &str) {
        // arrange

        // act
        let key = ConfigurationPath::section_key(path);

        // assert
        assert_eq!(key, expected);
    }

    #[test_case("", "" ; "when empty")]
    #[test_case(":::", "::" ; "when only delimiters")]
    #[test_case("a::b:::c", "a::b::" ; "with empty segments in the middle")]
    #[test_case("a:::b:", "a:::b" ; "when last segment is empty")]
    #[test_case("key", "" ; "with no parent")]
    #[test_case(":key", "" ; "with 1 empty parent")]
    #[test_case("::key", ":" ; "with 2 empty parents")]
    #[test_case("parent:key", "parent" ; "with parent")]
    fn parent_path_should_return_expected_segment(path: &str, expected: &str) {
        // arrange

        // act
        let key = ConfigurationPath::parent_path(path);

        // assert
        assert_eq!(key, expected);
    }
}
