use crate::cli::{ListOptions, Process};

impl Process for ListOptions {
    fn process(self) -> u32 {
        dbg!("process - list");

        return 0;
    }
}
