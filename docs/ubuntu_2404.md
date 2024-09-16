# Installing uvenv on Ubuntu 24.04

Explore three different methods to install `uvenv` on Ubuntu 24.04, each with its own approach, advantages, and steps.

## 1. Installing by Allowing System Package Modifications

You can install `uvenv` directly using `pip` with the `--break-system-packages` option. This will also install `uv` and
`patchelf`, which typically don't interfere with system packages. Alternatively, you can set the environment variable
`PIP_BREAK_SYSTEM_PACKAGES=1` to avoid issues with bash scripts from older Ubuntu versions such as 22.04.

**Why Choose This Method:**

- Quick and straightforward setup without additional tools.

**Considerations:**

- Minor risk of package conflicts, though unlikely with `uvenv`.

**How to Install:**

```bash
pip install --break-system-packages uvenv
```

## 2. Installing with pipx

If you prefer managing `uvenv` in an isolated environment and you already use `pipx` (`apt install pipx`), this can also
be a viable option. This approach keeps `uvenv` separate from your system’s Python packages.

**Benefits:**

- Keeps `uvenv` neatly isolated.
- Simplifies updates and removals without impacting other packages.

**Requirements:**

- `pipx` needs to be installed prior to this setup.

**Installation Steps:**

```bash
pipx install uvenv
```

## 3. Installing Inside a Virtual Environment

Creating a virtual environment specifically for `uvenv` ensures full isolation from system Python packages. This method
involves setting up a dedicated virtual environment and linking `uvenv` for easier access.

**Ideal For:**

- Those who want complete isolation from system Python packages.
- Users comfortable managing virtual environments.
- If you don't want to rely on `pipx` for management of `uvenv`

**Steps to Follow:**

```bash
python -m venv ~/.virtualenvs/uvenv  # Create the virtual environment
source ~/.virtualenvs/uvenv/bin/activate  # Activate the virtual environment
pip install uvenv  # Install uvenv within the virtual environment
uvenv setup  # Link uvenv to ~/.local/bin and ensure it’s in PATH
# Alternatively, for a simpler setup:
uvenv self link  # Create a symlink at ~/.local/bin/uvenv pointing to the current uvenv binary
```
