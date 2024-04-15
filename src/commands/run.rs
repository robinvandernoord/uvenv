use crate::cli::{Process, RunOptions};

impl Process for RunOptions {
    async fn process(self) -> Result<i32, String> {
        dbg!("process - run", self);
        // /tmp/bin
        return Ok(2);
    }
}
