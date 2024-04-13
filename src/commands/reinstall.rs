use crate::cli::{Process, ReinstallOptions};

impl Process for ReinstallOptions {
    async fn process(self) -> Result<u32, String> {
        dbg!("process - install");
        return Ok(2);
    }
}
