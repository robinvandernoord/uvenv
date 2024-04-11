use crate::cli::{Process, RunuvOptions};

impl Process for RunuvOptions {
    fn process(self) -> u32 {
        dbg!("process - runuv");
        return 0;
    }
}
