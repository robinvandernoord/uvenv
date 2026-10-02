use regex::Regex;

pub fn extract_python_version(input: &str) -> Option<String> {
    let Ok(re) = Regex::new(r"(\d+)\.(\d+)") else {
        return None;
    };

    re.captures(input).map(|caps| {
        let major = &caps[1];
        let minor = &caps[2];
        format!("{major}.{minor}")
    })
}
