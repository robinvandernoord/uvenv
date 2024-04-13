use crate::cli::{Process, UninstallOptions};

impl Process for UninstallOptions {
    async fn process(self) -> Result<u32, String> {
        dbg!("process - uninstall");
        return Ok(2);
    }
}
