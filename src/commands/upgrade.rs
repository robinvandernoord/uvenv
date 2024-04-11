use crate::cli::{Process, UpgradeOptions};

impl Process for UpgradeOptions {
    fn process(self) -> u32 {
        dbg!("process - upgrade");
        return 0;
    }
}
