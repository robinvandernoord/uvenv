use crate::cli::{InstallOptions, Process};

impl Process for InstallOptions {
    fn process(self) -> u32 {
        dbg!("process - install");
        return 0;
    }
}
