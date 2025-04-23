# Advanced Installation Options

Explore multiple alternative ways to install `uvenv` on systems where global pip installs are restricted (e.g., Ubuntu 24.04+).
Each method offers a different approach, with its own benefits and setup steps.

## 1. via `install.sh`

The easiest way to install `uvenv` is to use the [`install.sh`](https://github.com/robinvandernoord/uvenv/blob/uvenv/install.sh) script.

```bash
# download/read the script:
curl -fsSL https://raw.githubusercontent.com/robinvandernoord/uvenv/install-script/install.sh

# run it:
bash -c "$(curl -fsSL https://raw.githubusercontent.com/robinvandernoord/uvenv/install-script/install.sh)"
```


## 2. System Package Method

Install `uvenv` directly using `pip` with the `--break-system-packages` option.

**Advantages:**

- Quick and straightforward setup without additional tools.

**Considerations:**

- Minor risk of package conflicts, though unlikely with `uvenv`.

**Installation Steps:**

```bash
pip install --break-system-packages uvenv
```

## 3. Pipx Installation Method

Use `pipx` to manage `uvenv` in an isolated environment.

**Advantages:**

- Keeps `uvenv` isolated from system packages.
- Simplifies updates and removals.

**Prerequisites:**

- `pipx` must be installed (`apt install pipx`).

**Installation Steps:**

```bash
pipx install uvenv
```

## 4. Virtual Environment Method

Create a dedicated virtual environment for `uvenv`.

**Advantages:**

- Complete isolation from system Python packages.
- Suitable for users comfortable with virtual environments.

**Installation Steps:**

```bash
python -m venv ~/.virtualenvs/uvenv
source ~/.virtualenvs/uvenv/bin/activate
pip install uvenv
uvenv self link  # or `uvenv setup` for all additional features
```

## 5. Self-Managed uvenv Method

Use `uvenv` to manage its own installation and updates.

**Advantages:**

- Streamlines `uvenv` management through its own features.
- Simplifies long-term maintenance.

**Considerations:**

- Caution needed with commands like `uvenv uninstall-all`.

**Installation Steps:**

```bash
python -m venv /tmp/initial-uvenv
source /tmp/initial-uvenv/bin/activate
pip install uvenv
uvenv install uvenv
uvenv ensurepath  # or uvenv setup
```

## 6. via Snap

See [snap installation](./snap.md) for installation instructions and caveats.
