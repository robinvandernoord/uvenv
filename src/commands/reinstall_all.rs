use crate::cli::{Process, ReinstallAllOptions};

impl Process for ReinstallAllOptions {
    async fn process(self) -> Result<i32, String> {
        dbg!("process - reinstall-all", self);
        return Ok(2);
    }
}
