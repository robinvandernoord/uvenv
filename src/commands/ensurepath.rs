use crate::cli::{EnsurepathOptions, Process};

impl Process for EnsurepathOptions {
    fn process(self) -> u32 {
        dbg!("process - ensurepath");
        return 0;
    }
}
