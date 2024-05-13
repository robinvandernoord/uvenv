use crate::cli::{ActivateOptions, Process};

pub async fn install_activate() {
    // Used by `uvx --generate bash activate _`
    // note: only bash is supported right now!
    println!("{}", include_str!("../shell/activate.sh"));
}

impl Process for ActivateOptions {
    async fn process(self) -> Result<i32, String> {
        // wait a minute, this is not a bash script!
        // show warning with setup info:

        Ok(1)
    }
}

/*
declare -x VIRTUAL_ENV="/home/robin/rust/uvx/venv"
declare -x VIRTUAL_ENV_PROMPT="venv"
 */
