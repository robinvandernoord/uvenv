use crate::cli::{Process, ReinstallOptions};

impl Process for ReinstallOptions {
    fn process(self) -> u32 {
        dbg!("process - install");
        return 0;
    }
}
