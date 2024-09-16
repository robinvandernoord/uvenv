# uvenv: pipx for uv

Inspired by:

- [pipx](https://github.com/pypa/pipx)
- [uv](https://github.com/astral-sh/uv)

> **Note:** The tool previously named `uvx` is now `uvenv` due to a naming collision with a new `uv` command. The new name
> better reflects its purpose, combining `uv` with `venv`.  
> You can run `uvenv self migrate` to move your environments and installed commands from `uvx` to `uvenv`.

## Installation

1. Install via pip (or alternatives):
    ```bash
    pip install uvenv  
   # or `uv install uvenv`, `pipx install uvenv`
    ```

> Note: Ubuntu 24.04+ does not allow global installation via pip by default. 
> See [docs/ubuntu_2404.md](./docs/ubuntu_2404.md) if you encounter `externally-managed-environment` errors.

2. Optional (for bash users):
      ```bash
      uvenv setup
      ```

   This installs the following features:

- Ensures `~/.local/bin/` is added to your PATH, so commands can be found (unless `--skip-ensurepath`). Can also be
  activated via `uvenv ensurepath`
- Enables tab completion for `uvenv` (unless `--skip-completions`). Can also be enabled
  via `uvenv completions --install`.
- Enables `uvenv activate` (unless `--skip-activate`) to activate uvenv-managed virtualenvs from your shell

## Usage

```bash
uvenv
```

Run `uvenv` without any arguments to see all possible subcommands.

## Platform Considerations

- **Rust-Powered Performance (uvenv 2.0):** Starting from version 2.0, `uvenv` leverages Rust for improved performance
  and
  compatibility with `uv`.
- **Prebuilt Binaries:** Currently, prebuilt binaries are available for x86_64 (amd64) and aarch64 (ARM64) on Linux.
- **Other Platforms:** If you're on a different platform, you can still use `uvx 1.x`, which is written in pure
  Python.
  Find it at [robinvandernoord/uvx](https://github.com/robinvandernoord/uvx).
- Alternatively, you can **Compile for Your Platform**:
    - Install the Rust toolchain:
        ```bash
        curl https://sh.rustup.rs -sSf | sh
        ```
    - Clone the `uvenv` repo and navigate to it:
        ```bash
        git clone https://github.com/robinvandernoord/uvenv.git
        cd uvenv
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
    - Compile and install the `uvenv` binary:
        ```bash
        maturin develop
        ```
    - Now you can use `uvenv`:
        ```bash
        ./venv/bin/uvenv
        ```

For additional details on building and distribution, refer to [maturin](https://www.maturin.rs/distribution)
documentation.

## License

`uvenv` is distributed under the terms of the [MIT](https://spdx.org/licenses/MIT.html) license.

## Changelog

See `CHANGELOG.md` [on GitHub](https://github.com/robinvandernoord/uvenv/blob/master/CHANGELOG.md)
