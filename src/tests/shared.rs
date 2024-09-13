use crate::metadata::get_home_dir;
use core::error::Error;
use std::path::Path;

pub type TestResult = Result<(), Box<dyn Error>>;

pub fn is_empty(some_dir: &Path) -> bool {
    let Ok(mut dir) = some_dir.read_dir() else {
        return false;
    };

    dir.next().is_none()
}

pub fn cleanup() {
    // every get_home_dir should clean it up (in test mode)
    let home_dir = get_home_dir();
    assert!(home_dir.exists(), "Home should exist");
    assert!(is_empty(&home_dir), "Home should be empty");
}
