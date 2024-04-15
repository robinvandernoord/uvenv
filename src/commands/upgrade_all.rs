use crate::cli::{Process, UpgradeAllOptions};

impl Process for UpgradeAllOptions {
    async fn process(self) -> Result<i32, String> {
        dbg!("process - upgrade-all", self);
        return Ok(2);
    }
}
