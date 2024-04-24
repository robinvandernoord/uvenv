# uvx: pipx for uv

Inspired by:

- [pipx](https://github.com/pypa/pipx)
- [uv](https://github.com/astral-sh/uv)

## Installation

```bash
# one of these ways:
pip install uvx # or `uv`, `pipx`

# optional (in bash):
uvx ensurepath # make sure ~/.local/bin is in PATH
uvx completions --install # enable tab completion
```

### Platforms

Since `uvx 2.0`, this tool uses Rust for performance and compatibility with `uv`.
Currently, only prebuilt binaries are available for x86_64 (amd64) and aarch64 (ARM64) on Linux.
Other platforms can use `uvx 1.x`, which is written in pure Python and can be found
at [robinvandernoord/uvx](https://github.com/robinvandernoord/uvx).
You can also compile `uvx` for your own platform:

```bash
# install the rust toolchain:
curl https://sh.rustup.rs -sSf | sh
# clone the repo and enter it:
git clone https://github.com/robinvandernoord/uvx2.git; 
cd uvx2;
# install a virtualenv (choose python or uv)
python -m venv venv  # or `uv venv venv --seed`
source venv/bin/activate

# install maturin
pip install maturin # uv pip install maturin

# compile and install the binary:
maturin develop

# uvx is now available:
./venv/bin/uvx
```

For more info about building and distribution, see [maturin](https://www.maturin.rs/distribution).

## Usage

```bash
uvx
```

Run `uvx` without any arguments to see all possible subcommands.

## License

`uvx` is distributed under the terms of the [MIT](https://spdx.org/licenses/MIT.html) license.

## Changelog

See `CHANGELOG.md` [on GitHub](https://github.com/robinvandernoord/uvx2/blob/master/CHANGELOG.md)
