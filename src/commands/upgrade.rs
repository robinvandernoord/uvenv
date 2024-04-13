use crate::cli::{Process, UpgradeOptions};

impl Process for UpgradeOptions {
    async fn process(self) -> Result<u32, String> {
        dbg!("process - upgrade");
        return Ok(2);
    }
}
