# uvenv: pipx for uv

![pypi wheels](https://github.com/robinvandernoord/uvenv/actions/workflows/pypi.yml/badge.svg)
![snapcraft](https://github.com/robinvandernoord/uvenv/actions/workflows/snap.yml/badge.svg)


Inspired by:

- [pipx](https://github.com/pypa/pipx)
- [uv](https://github.com/astral-sh/uv)

## Installation

> **New**: `uvenv` is now also installable via the snap store.  
> The `pip` method is still recommended, but if you want to use `snap`, please check out [docs/snap.md](./docs/snap.md)!

1. Install via pip (or alternatives):
    ```bash
    pip install uvenv  
   # or `uv install uvenv`, `pipx install uvenv`
    ```

> Note: On some systems (e.g., Ubuntu 24.04+), global installation via pip is restricted by default.
> The recommended way to install `uvenv` for these systems is to use the [`install.sh`](https://github.com/robinvandernoord/uvenv/blob/uvenv/install.sh) script:
> ```bash
> $SHELL -c "$(curl -fsSL https://raw.githubusercontent.com/robinvandernoord/uvenv/uvenv/install.sh)"
> # instead of `$SHELL`, you can also use `sh`, `bash`, `zsh`
> > ```
> For more installation alternatives, see [docs/installation.md](docs/installation.md) if you encounter `externally-managed-environment` errors.

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

Most `pipx` commands are supported, such as `install`, `upgrade` `inject`, `run`, `runpip`.  
Run `uvenv` without any arguments to see all possible subcommands.

### 🆕 Freeze and Thaw

You can snapshot your current setup into a `uvenv.lock` file using:

```bash
uvenv freeze
```

This lock file records all installed applications along with their metadata — including version, Python version, and any injected dependencies.

Later, you can restore that exact setup using:

```bash
uvenv thaw
```

This is useful for replicating the same setup on a different machine, or after a clean install or system update.

#### Lock file formats

The `uvenv.lock` file can be saved in one of the following formats:

- **TOML** (default): human-readable and easy to edit
- **JSON**: more verbose, but script-friendly (e.g. with `jq`)
- **Binary**: compact, but not human-readable

Choose the format using the `--format` flag:

```bash
uvenv freeze --format json
```

See [docs/lockfile_v1.md](./docs/lockfile_v1.md) for details on the file format, including all supported options and examples.

#### Selective freeze/thaw

Use `--include` or `--exclude` to control which apps get recorded or restored:

```bash
uvenv freeze --exclude some-app
uvenv thaw --include only-this-app
```

For all available options, see:

```bash
uvenv freeze --help
uvenv thaw --help
```

## Migrating from `uvx` and Comparing with `uv tool`

### Migrating from `uvx`

The tool previously named `uvx` is now `uvenv` due to a naming collision with a new `uv` command. The new name better reflects its purpose, combining `uv` with `venv`.
You can run `uvenv self migrate` to move your environments and installed commands from `uvx` to `uvenv`.

---

### How `uvenv` differs from `uv tool`

While both `uvenv` and `uv tool` (a subcommand of [`uv`](https://github.com/astral-sh/uv)) offer overlapping functionality for installing and running Python applications, they differ in purpose and approach:

* **Interface:** `uvenv` is modeled after `pipx`, offering commands like `install`, `inject`, `run`, `upgrade`, and `runpip`. If you're already used to `pipx`, `uvenv` is a near drop-in replacement.
* **Inject support:** `uvenv` supports `pipx`'s `inject` functionality, which lets you add extra packages to an app’s environment — helpful for plugins, linters, or testing tools. `uv tool` does not currently support this.
* **Compatibility:** `uvenv` uses `uv` for dependency resolution and installation, benefiting from its speed and correctness. It also respects `uv`'s configuration files (such as `~/.config/uv/uv.toml` and `/etc/uv/uv.toml`, see [uv config docs](https://docs.astral.sh/uv/configuration/files/)) unless the environment variable `UV_NO_CONFIG=1` is set to ignore them.

In short:

* Use **`uvenv`** if you want `pipx`-style workflows with advanced management features.
* Use **`uv tool`** if you prefer a minimal approach for running tools quickly - for most basic use-cases, `uv tool` is probably sufficient.


## Platform Considerations

- **Rust-Powered Performance (uvenv 2.0):** Starting from version 2.0, `uvenv` leverages Rust for improved performance
  and compatibility with `uv`.
- **Prebuilt Binaries:** Currently, prebuilt binaries are available for x86_64 (amd64) and aarch64 (ARM64) on Linux, as well as Intel (x86_64) and Apple Silicon (ARM64) on macOS.
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
