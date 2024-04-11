use crate::cli::{Process, RunpipOptions};

impl Process for RunpipOptions {
    fn process(self) -> u32 {
        dbg!("process - runpip");
        return 0;
    }
}
