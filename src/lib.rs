#![doc = include_str!("../README.md")]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod builder;
mod configuration;
mod error;
mod file;
mod merge;
mod provider;
mod section;
mod settings;

/// Contains chained configuration support.
#[cfg(feature = "chained")]
pub mod chained;

/// Contains command line configuration support.
#[cfg(feature = "cmd")]
pub mod cmd;

/// Contains strongly-typed configuration deserialization support.
#[cfg(feature = "binder")]
pub mod de;

/// Contains environment variable configuration support.
#[cfg(feature = "env")]
pub mod env;

/// Contains `*.ini` file configuration support.
#[cfg(feature = "ini")]
pub mod ini;

/// Contains `*.json` file configuration support.
#[cfg(feature = "json")]
pub mod json;

/// Contains in-memory configuration support.
#[cfg(feature = "mem")]
pub mod mem;

/// Provides configuration path utilities.
pub mod path;

/// Contains library prelude.
pub mod prelude;

/// Contains `*.xml` file configuration support.
#[cfg(feature = "xml")]
pub mod xml;

/// Contains `*.yaml` and `*.yml` file configuration support.
#[cfg(feature = "yaml")]
pub mod yaml;

pub use builder::Builder;
pub use configuration::Configuration;
pub use error::Error;
pub use file::{FileSource, FileSourceBuilder};
pub use merge::Merge;
pub use provider::Provider;
pub use section::Section;
pub use settings::Settings;

/// Represents the type alias for a configuration reference.
#[cfg(not(feature = "async"))]
pub type Ref<T> = std::rc::Rc<T>;

/// Represents the type alias for a configuration reference.
#[cfg(feature = "async")]
pub type Ref<T> = std::sync::Arc<T>;

/// Represents a configuration result.
pub type Result<T = ()> = std::result::Result<T, Error>;

/// Creates and returns a new [configuration builder](Builder)
#[inline]
pub fn builder() -> Builder {
    Builder::default()
}

/// Converts the specified text into Pascal Case.
///
/// # Arguments
///
/// * `text` - the input text to convert
///
/// # Remarks
///
/// This function supports converting the following input forms:
///
/// - Pascal Case (`HelloWorld → HelloWorld`)
/// - Camel Case (`helloWorld → HelloWorld`)
/// - Snake Case (`hello_world → HelloWorld`)
/// - Screaming Snake Case (`HELLO_WORLD → HelloWorld`)
/// - Kebab Case (`hello-world → HelloWorld`)
/// - Screaming Kebab Case (`HELLO-WORLD → HelloWorld`)
///
/// The characters `' '`, `'_'`, and `'-'` are considered word boundaries. Alphabetic characters following these
/// characters will be capitalized.
pub fn pascal_case(text: &str) -> String {
    let mut converted = String::with_capacity(text.len());
    let mut next_is_upper = true;
    let mut last_was_lower = false;

    for ch in text.chars() {
        if ch == ' ' || ch == '_' || ch == '-' || ch == ':' {
            next_is_upper = true;
            last_was_lower = false;

            if ch == ':' {
                converted.push(ch);
            }
        } else if ch.is_alphabetic() && (next_is_upper || (last_was_lower && ch.is_ascii_uppercase())) {
            converted.push(ch.to_ascii_uppercase());
            next_is_upper = false;
            last_was_lower = ch.is_ascii_lowercase();
        } else if ch.is_alphabetic() {
            converted.push(ch.to_ascii_lowercase());
            last_was_lower = ch.is_ascii_lowercase();
        } else {
            converted.push(ch);
            next_is_upper = true;
            last_was_lower = false;
        }
    }

    converted
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case(""; "if empty")]
    #[test_case("HelloWorld"; "in pascal case")]
    #[test_case("Hello.World"; "with a period")]
    #[test_case("Hello:World"; "with a colon")]
    fn pascal_case_should_not_change_text(expected: &str) {
        // arrange

        // act
        let actual = pascal_case(expected);

        // assert
        assert_eq!(actual, expected);
    }

    #[test_case("hello world"; "from lower title case")]
    #[test_case("Hello World"; "from upper title case")]
    #[test_case("helloWorld"; "from camel case")]
    #[test_case("hello_world"; "from snake case")]
    #[test_case("HELLO_WORLD"; "from screaming snake case")]
    #[test_case("hello-world"; "from kebab case")]
    #[test_case("HELLO-WORLD"; "from screaming kebab case")]
    fn pascal_case_should_convert_text(text: &str) {
        // arrange

        // act
        let actual = pascal_case(text);

        // assert
        assert_eq!(actual, "HelloWorld");
    }

    #[test]
    fn pascal_case_should_convert_capitalized_with_colon() {
        // arrange
        let expected = "Hello:World";

        // act
        let actual = pascal_case("HELLO:WORLD");

        // assert
        assert_eq!(actual, expected);
    }
}
