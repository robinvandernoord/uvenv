use crate::cli::{Process, RunpythonOptions};

impl Process for RunpythonOptions {
    fn process(self) -> Result<u32, String> {
        dbg!("process - runpython");
        return Ok(2);
    }
}
