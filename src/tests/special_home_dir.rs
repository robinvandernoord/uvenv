use std::fs;
use std::fs::File;
use std::io::Write;

use crate::helpers::PathToString;
use crate::metadata::get_home_dir;
use crate::tests::shared;

fn test_0_custom_home_dir() {
    let home_dir_path = get_home_dir();
    let home_dir_str = home_dir_path.clone().to_string();

    assert!(
        !home_dir_str.contains(".local"),
        ".local shouldn't exist yet"
    );
    assert!(
        home_dir_str.starts_with("/tmp/uvenv-test"),
        "Home should live at /tmp for tests!"
    );

    assert!(home_dir_path.exists(), "Home should exist!");
    assert!(shared::is_empty(&home_dir_path), "Home should be empty!");
}

#[expect(clippy::panic_in_result_fn, reason = "This is a test file.")]
fn test_1_write_file() -> shared::TestResult {
    let home = get_home_dir();
    let file_path = home.join("assert_write_file");

    // Write to the file
    let mut file = File::create(&file_path)?;
    let content = b"Hello, Rust!";
    file.write_all(content)?;

    // Read from the file
    // let mut file = File::open(&file_path)?;
    // let mut buffer = Vec::new();
    // file.read_to_end(&mut buffer)?;

    let buffer = fs::read(&file_path)?;

    // Assert that the content is as expected
    assert_eq!(
        buffer, content,
        "loaded file contents should the same as was written"
    );

    assert!(!shared::is_empty(&home), "Home should not be empty!");

    Ok(())
}

#[test]
/// special test which makes sure uvenv uses a custom home directory
/// to prevent breaking normal installed uvenv packages on host system.
fn test_home_dir_flow() -> shared::TestResult {
    test_0_custom_home_dir();
    test_1_write_file()?;
    shared::cleanup();

    Ok(())
}
