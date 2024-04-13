use crate::cli::{Process, SelfUpdateOptions};

impl Process for SelfUpdateOptions {
    async fn process(self) -> Result<u32, String> {
        dbg!("process - self-update");
        return Ok(2);
    }
}
