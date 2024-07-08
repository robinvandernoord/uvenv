use std::fs::File;
use std::io::{Read, Write};

use crate::helpers::PathToString;
use crate::metadata::get_home_dir;
use crate::tests::shared;

fn test_0_custom_home_dir() {
    let home_dir_path = get_home_dir();
    let home_dir_str = home_dir_path.clone().to_string();

    assert!(!home_dir_str.contains(".local"));
    assert!(home_dir_str.starts_with("/tmp/uvenv-test"));

    assert!(home_dir_path.exists());
    assert!(shared::is_empty(&home_dir_path));
}

fn test_1_write_file() -> shared::TestResult {
    let home = get_home_dir();
    let file_path = home.join("assert_write_file");

    // Write to the file
    let mut file = File::create(&file_path)?;
    let content = b"Hello, Rust!";
    file.write_all(content)?;

    // Read from the file
    let mut file = File::open(&file_path)?;
    let mut buffer = Vec::new();
    file.read_to_end(&mut buffer)?;

    // Assert that the content is as expected
    assert_eq!(buffer, content);

    assert!(!shared::is_empty(&home));

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
