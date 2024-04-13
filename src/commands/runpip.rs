use crate::cli::{Process, RunpipOptions};

impl Process for RunpipOptions {
    async fn process(self) -> Result<u32, String> {
        dbg!("process - runpip");
        return Ok(2);
    }
}
