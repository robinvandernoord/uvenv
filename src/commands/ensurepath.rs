use crate::cli::{EnsurepathOptions, Process};

impl Process for EnsurepathOptions {
    async fn process(self) -> Result<u32, String> {
        dbg!("process - ensurepath");
        return Ok(2);
    }
}
