use crate::cli::{Process, SelfUpdateOptions};

impl Process for SelfUpdateOptions {
    fn process(self) -> u32 {
        dbg!("process - self-update");
        return 0;
    }
}
