use crate::cli::{InjectOptions, Process};

impl Process for InjectOptions {
    fn process(self) -> u32 {
        dbg!("process - inject");
        return 0;
    }
}
