use crate::cli::{Process, RunpythonOptions};

impl Process for RunpythonOptions {
    async fn process(self) -> Result<i32, String> {
        dbg!("process - runpython", self);
        return Ok(2);
    }
}
