use crate::cli::{InstallOptions, Process};

impl Process for InstallOptions {
    fn process(self) -> Result<u32, String> {
        dbg!("process - install");
        return Ok(2);
    }
}
