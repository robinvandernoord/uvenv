use crate::cli::{Process, UnInjectOptions};

impl Process for UnInjectOptions {
    async fn process(self) -> Result<i32, String> {
        dbg!("process - uninject", self);
        return Ok(2);
    }
}
