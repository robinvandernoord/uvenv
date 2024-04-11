use crate::cli::{Process, UpgradeAllOptions};

impl Process for UpgradeAllOptions {
    fn process(self) -> u32 {
        dbg!("process - upgrade-all");
        return 0;
    }
}
