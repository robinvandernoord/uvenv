use crate::cli::{Process, SelfUpdateOptions};

impl Process for SelfUpdateOptions {
    async fn process(self) -> Result<i32, String> {
        dbg!("process - self-update", self);
        return Ok(2);
    }
}
