mod cli;
mod commands;

use crate::cli::{Args, Process};
use pyo3::prelude as pyo;

async fn async_main_rs() -> u32 {
    // let args = Args::parse_from(env::args().skip(1)); // first argument is now 'python' instead of 'uvx' so skip it
    let args = Args::parse_from_python();

    args.cmd.process()
}

#[pyo::pyfunction]
fn main_rs(py: pyo::Python<'_>) -> pyo::PyResult<&pyo::PyAny> {
    pyo3_asyncio::tokio::future_into_py(py, async {
        let exit_code = async_main_rs().await;

        Ok(pyo::Python::with_gil(|_| exit_code))
    })
}

/// A Python module implemented in Rust.
#[pyo::pymodule]
fn uvx(_py: pyo::Python, m: &pyo::PyModule) -> pyo::PyResult<()> {
    m.add_function(pyo::wrap_pyfunction!(main_rs, m)?)?;
    Ok(())
}
