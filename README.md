
# Deprecation Warning

[https://pypi.org/project/uvenv](https://pypi.org/project/uvenv)

> **Note:** The tool previously named `uvx` is now `uvenv` due to a naming collision with a new `uv` command. The new name
> better reflects its purpose, combining `uv` with `venv`.  
> You can run `uvenv self migrate` to move your environments and installed commands from `uvx` to `uvenv`.


---

---

---

# uvx: pipx for uv

Inspired by:

- [pipx](https://github.com/pypa/pipx)
- [uv](https://github.com/astral-sh/uv)

## Installation

1. Install via pip (or alternatives):
    ```bash
    pip install uvx  # or `uv`, `pipx`
    ```

2. Optional (for bash users):
      ```bash
      uvx setup
      ```

   This installs the following features:

- Ensures `~/.local/bin/` is added to your PATH, so commands can be found (unless `--skip-ensurepath`). Can also be
  activated via `uvx ensurepath`
- Enables tab completion for `uvx` (unless `--skip-completions`). Can also be enabled via `uvx completions --install`.
- Enables `uvx activate` (unless `--skip-activate`) to activate uvx-managed virtualenvs from your shell

## Usage

```bash
uvx
```

Run `uvx` without any arguments to see all possible subcommands.

## Platform Considerations

- **Rust-Powered Performance (uvx 2.0):** Starting from version 2.0, `uvx` leverages Rust for improved performance and
  compatibility with `uv`.
- **Prebuilt Binaries:** Currently, prebuilt binaries are available for x86_64 (amd64) and aarch64 (ARM64) on Linux.
- **Other Platforms:** If you're on a different platform, you can still use `uvx 1.x`, which is written in pure Python.
  Find it at [robinvandernoord/uvx](https://github.com/robinvandernoord/uvx).
- Alternatively, you can **Compile for Your Platform**:
    - Install the Rust toolchain:
        ```bash
        curl https://sh.rustup.rs -sSf | sh
        ```
    - Clone the `uvx2` repo and navigate to it:
        ```bash
        git clone https://github.com/robinvandernoord/uvx2.git
        cd uvx2
        ```
    - Set up a virtual environment (choose Python or uv):
        ```bash
        python -m venv venv  # or `uv venv venv --seed`
        source venv/bin/activate
        ```
    - Install Maturin (Python with Rust package builder):
        ```bash
        pip install maturin  # or `uv pip install maturin`
        ```
    - Compile and install the `uvx` binary:
        ```bash
        maturin develop
        ```
    - Now you can use `uvx`:
        ```bash
        ./venv/bin/uvx
        ```

For additional details on building and distribution, refer to [maturin](https://www.maturin.rs/distribution)
documentation.

## License

`uvx` is distributed under the terms of the [MIT](https://spdx.org/licenses/MIT.html) license.

## Changelog

See `CHANGELOG.md` [on GitHub](https://github.com/robinvandernoord/uvx2/blob/master/CHANGELOG.md)
