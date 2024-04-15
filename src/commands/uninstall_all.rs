use crate::cli::{Process, UninstallAllOptions};

impl Process for UninstallAllOptions {
    async fn process(self) -> Result<i32, String> {
        dbg!("process - uninstall-all", self);
        return Ok(2);
    }
}
