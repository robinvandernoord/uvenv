use crate::cli::{Process, UninstallOptions};

impl Process for UninstallOptions {
    fn process(self) -> u32 {
        dbg!("process - uninstall");
        return 0;
    }
}
