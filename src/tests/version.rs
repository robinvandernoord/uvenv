#[allow(unused_imports)]
use crate::commands::self_version::_compare_versions;
#[allow(unused_imports)]
use crate::tests::shared::TestResult;

#[test]
/// special test which makes sure uvenv uses a custom home directory
/// to prevent breaking normal installed uvenv packages on host system.
fn test_is_latest() -> TestResult {
    assert!(_compare_versions("1.2.3", "1.2.3"));
    assert!(_compare_versions("1.3.3", "1.2.3"));
    assert!(!_compare_versions("1.2.3", "1.3.3"));

    Ok(())
}
