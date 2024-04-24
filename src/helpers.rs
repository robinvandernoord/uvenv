use std::path::PathBuf;

pub trait ResultToString<T, E> {
    fn map_err_to_string(self) -> Result<T, String>;
}

impl<T, E: std::error::Error> ResultToString<T, E> for Result<T, E> {
    fn map_err_to_string(self) -> Result<T, String> {
        self.map_err(|e| e.to_string())
    }
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

pub trait PathToString {
    fn to_string(self) -> String;
}

impl PathToString for PathBuf {
    fn to_string(self) -> String {
        self.into_os_string().into_string().unwrap_or_default()
    }
}
