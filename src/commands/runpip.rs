use crate::cli::{Process, RunpipOptions};

impl Process for RunpipOptions {
    fn process(self) -> Result<u32, String> {
        dbg!("process - runpip");
        return Ok(2);
    }
}
