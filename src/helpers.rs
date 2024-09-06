use std::path::{Path, PathBuf};

pub trait ResultToString<T, E> {
    #[allow(dead_code)]
    fn map_err_to_string(self) -> Result<T, String>;
}

impl<T, E: std::error::Error> ResultToString<T, E> for Result<T, E> {
    fn map_err_to_string(self) -> Result<T, String> {
        self.map_err(|e| e.to_string())
    }
}

pub fn fmt_error(e: &anyhow::Error) -> String {
    format!("{e:?}")
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
        let mut s = self.into();
        if s.is_empty() {
            s.push_str(dflt);
        }
        s
    }
}

pub trait PathAsStr<'a> {
    fn as_str(&'a self) -> &'a str;
}

impl<'a> PathAsStr<'a> for PathBuf {
    fn as_str(&'a self) -> &'a str {
        self.to_str().unwrap_or_default()
    }
}

impl<'a> PathAsStr<'a> for Path {
    fn as_str(&'a self) -> &'a str {
        self.to_str().unwrap_or_default()
    }
}

pub trait PathToString<'a>: PathAsStr<'a> {
    fn to_string(self) -> String;
}

/// `PathToString` can't be implemented for Path because sizes need to be known at comptime
impl<'a> PathToString<'a> for PathBuf {
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
