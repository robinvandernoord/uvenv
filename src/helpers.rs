use core::any::type_name;
use std::path::{Path, PathBuf};

pub trait ResultToString<T, E> {
    #[expect(dead_code, reason = "Could still be useful sometimes")]
    fn map_err_to_string(self) -> Result<T, String>;
}

impl<T, E: core::error::Error> ResultToString<T, E> for Result<T, E> {
    fn map_err_to_string(self) -> Result<T, String> {
        self.map_err(|err| err.to_string())
    }
}

pub fn fmt_error(err: &anyhow::Error) -> String {
    format!("{err:?}")
}

/// Source: <https://users.rust-lang.org/t/how-to-print-the-type-of-a-variable/101947/2>
#[expect(dead_code, clippy::use_debug, reason = "Debugging reasons.")]
pub fn print_type<T>(_: &T) {
    println!("{:?}", type_name::<T>());
}

// https://users.rust-lang.org/t/is-there-a-simple-way-to-give-a-default-string-if-the-string-variable-is-empty/100411

pub trait StringExt {
    fn or(
        self,
        dflt: &str,
    ) -> String;
}

impl<S: Into<String>> StringExt for S {
    fn or(
        self,
        dflt: &str,
    ) -> String {
        // Re-use a `String`s capacity, maybe
        let mut result_string = self.into();
        if result_string.is_empty() {
            result_string.push_str(dflt);
        }
        result_string
    }
}

pub trait PathAsStr<'path> {
    fn as_str(&'path self) -> &'path str;
}

impl<'path> PathAsStr<'path> for PathBuf {
    fn as_str(&'path self) -> &'path str {
        self.to_str().unwrap_or_default()
    }
}

impl<'path> PathAsStr<'path> for Path {
    fn as_str(&'path self) -> &'path str {
        self.to_str().unwrap_or_default()
    }
}

pub trait PathToString<'path>: PathAsStr<'path> {
    fn to_string(self) -> String;
}

/// `PathToString` can't be implemented for Path because sizes need to be known at comptime
impl<'path> PathToString<'path> for PathBuf {
    fn to_string(self) -> String {
        self.into_os_string().into_string().unwrap_or_default()
    }
}

/// `Option<Option<T>>` can be flattened with `.flatten()`
/// but this can be used for Option<&Option<T>>
pub const fn flatten_option_ref<T>(nested: Option<&Option<T>>) -> Option<&T> {
    match nested {
        Some(Some(version)) => Some(version),
        _ => None,
    }
}
