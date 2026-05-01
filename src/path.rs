use std::cmp::Ordering::{self, *};

/// Gets the key delimiter used in configuration paths.
pub const fn delimiter() -> char {
    ':'
}

/// Combines the specified segments into one path.
///
/// # Arguments
///
/// * `segments` - The segments to combine into a path
#[inline]
pub fn combine(segments: &[&str]) -> String {
    segments.join(":")
}

/// Extracts the last path segment from the path.
///
/// # Arguments
///
/// * `path` - The path to extract the key from
pub fn last(path: &str) -> &str {
    if let Some(index) = path.rfind(delimiter()) {
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
pub fn parent(path: &str) -> &str {
    if let Some(index) = path.rfind(delimiter()) {
        &path[..index]
    } else {
        ""
    }
}

/// Extracts the next path segment with a given base.
///
/// # Arguments
///
/// * `path` - The path to extract the key from
/// * `base` - The optional base path to match against
pub fn next<'a>(path: &'a str, base: Option<&str>) -> Option<&'a str> {
    if path.is_empty() {
        None
    } else if let Some(base) = base {
        let len = base.len();

        if path.len() > len && path[..len].eq_ignore_ascii_case(base) && path.chars().nth(len) == Some(delimiter()) {
            if let Some(index) = path[(len + 1)..].find(delimiter()) {
                Some(&path[..(len + 1 + index)])
            } else {
                Some(path)
            }
        } else {
            None
        }
    } else if let Some(index) = path.find(delimiter()) {
        Some(&path[..index])
    } else {
        Some(path)
    }
}

/// Compares two key paths.
///
/// # Arguments
///
/// * `lhs` - The left-hand side to compare
/// * `rhs` - The right-hand side to compare against
pub fn cmp<S: AsRef<str>>(lhs: &S, rhs: &S) -> Ordering {
    let lhs = lhs.as_ref();
    let rhs = rhs.as_ref();
    let a = lhs.split(delimiter()).filter(|s| !s.is_empty()).count();
    let b = rhs.split(delimiter()).filter(|s| !s.is_empty()).count();
    let mut result = a.cmp(&b);

    if result != Equal {
        return result;
    }

    let segments = lhs
        .split(delimiter())
        .filter(|s| !s.is_empty())
        .zip(rhs.split(delimiter()).filter(|s| !s.is_empty()));

    for (a, b) in segments {
        if let Ok(x) = a.parse::<usize>() {
            if let Ok(y) = b.parse::<usize>() {
                // int : int
                result = x.cmp(&y);

                if result != Equal {
                    return result;
                }
            } else {
                // int : string
                return Less;
            }
        } else if b.parse::<usize>().is_ok() {
            // string : int
            return Greater;
        } else {
            // string : string
            result = a.to_uppercase().cmp(&b.to_uppercase());

            if result != Equal {
                return result;
            }
        }
    }

    Equal
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
        let path = combine(segments);

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
    fn last_should_return_expected_segment(path: &str, expected: &str) {
        // arrange

        // act
        let key = last(path);

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
    fn parent_should_return_expected_segment(path: &str, expected: &str) {
        // arrange

        // act
        let key = parent(path);

        // assert
        assert_eq!(key, expected);
    }

    #[test_case("a", Some("") ; "when empty")]
    #[test_case("a", Some("a:b") ; "when path is too short")]
    #[test_case("a:b", Some("a:b") ; "when path and base are equal")]
    fn next_should_return_none(path: &str, base: Option<&str>) {
        // arrange

        // act
        let key = next(path, base);

        // assert
        assert_eq!(key, None);
    }

    #[test_case("a:b", Some("a"), Some("a:b") ; "when base has 1 segment")]
    #[test_case("a:b:c", Some("a:b"), Some("a:b:c") ; "when base has 2 segments")]
    #[test_case("a:b:c", Some("a"), Some("a:b") ; "when path has 3 segments")]
    fn next_should_return_some(path: &str, base: Option<&str>, expected: Option<&str>) {
        // arrange

        // act
        let key = next(path, base);

        // assert
        assert_eq!(key, expected);
    }
}
