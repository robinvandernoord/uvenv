use crate::commands::self_info::compare_versions;

#[test]
fn is_latest() {
    assert!(compare_versions("1.2.3", "1.2.3"));
    assert!(compare_versions("1.3.3", "1.2.3"));
    assert!(compare_versions("1.2.10", "1.2.3"));
    assert!(!compare_versions("1.2.3", "1.3.3"));
    assert!(!compare_versions("1.3.3", "1.3.13"));
}
