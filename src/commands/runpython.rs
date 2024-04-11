use crate::cli::{Process, RunpythonOptions};

impl Process for RunpythonOptions {
    fn process(self) -> u32 {
        dbg!("process - runpython");
        return 0;
    }
}
