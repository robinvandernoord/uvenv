# Changelog

<!--next-version-placeholder-->

## 3.4.2 (2024-10-10)

### Updates

* upgrade to `uv` 0.4.20

## 3.4.1 (2024-10-08)

### Updates

* upgrade to `uv` 0.4.19

## 3.4.0 (2024-09-16)

### Feature

* `uvenv self link` to create a symlink to `~/.local/bin/uvenv` - which is useful when you installed `uvenv` in a
  separate virtualenv.

### Docs

* Described multiple ways of installing `uvenv` in Ubuntu 24.04 (for `externally-managed-environment` errors)

## 3.3.6 (2024-09-16)

### Fix

* improved error messages for `ensurepath`
* bump dependencies (uv to 0.4.10)

## 3.3.5 (2024-09-06)

### Fix

* rollback to pip-only self update (but keep new functions for documentation purposes)
* exclude yanked versions from 'latest' version

## 3.3.4 (2024-09-04)

### Fix

* rollback slightly: still use `pip freeze` as backup

## 3.3.3 (2024-09-04)

### Fix

* use `uv` for `self update` if possible, replaced pip_freeze with uv_freeze

## 3.3.2 (2024-09-04)

### Updates

* bump dependencies (uv 0.4.4)

## 3.3.1 (2024-08-13)

### Fix

* replace --break-system-packages with environment variable for backwards compatibility

## 3.3.0 (2024-08-13)

### Feature

* allow --python in `uvenv list` to filter installed packages by py version

## 3.2.2 (2024-08-07)

### Fix

* don't say uvenv is outdated when it's actually ahead of the pypi version (bc caching)

## 3.2.1 (2024-08-07)

### Fix

* `uvenv list` should NOT stop after displaying outdated message

## 3.2.0 (2024-08-07)

### Features

* add `uvenv self version` to show version info about uvenv and its dependencies
* warn about outdated uvenv on 'uvenv list'

### Fix

* allow `self update` on ubuntu 24.04 by setting --break-system-packages
* fix changelog headings
* use new default branch 'uvenv' instead of 'master' for changelog

## 3.1.1 (2024-07-20)

### Fix

* Upgrade to work with uv 0.2.27

## 3.1.0 (2024-07-16)

### Features

- `--with` for install and run to immediately inject dependencies

### Fix

- make sure uv cache is available when venv is activated

### Updates

- bump to uv 0.2.25
- update other cargo dependencies

## 3.0.2 (2024-07-10)

### Fix

- Improved changelog parsing

## 3.0.1 (2024-07-10)

### Updates

- Bump `uv` to 0.2.24

## 3.0.0 (2024-07-08)

### BREAKING CHANGE

- **Renaming**: Renamed `uvx` to `uvenv` due to a naming collision with a new `uv` command. The
  new name better reflects its purpose, combining `uv` with `venv`. You can run `uvenv self migrate` to move your
  environments and installed commands from `uvx` to `uvenv`.

### Features

- Added `uvenv self migrate` command to facilitate migration from `uvx` to `uvenv` easily.
- Improved error logging (with more context) using `anyhow`.

### Updates

- Updated `uv` from 0.2.4 to 0.2.13 and applied necessary patches to work with new/updated APIs.

### Documentation

- Updated documentation to reflect the changes and new features introduced in this version.
- Started groundwork on automated testing

## 2.5.1 (2024-07-20)

### Fix

* Upgrade to work with uv 0.2.27

## 2.5.0 (2024-07-08)

### Feature

* Show deprecation warning in favor of `uvenv`
* Migrate to uvenv on `self update`

## 2.4.1 (2024-05-31)

### Fix

* Rollback some of the speedup (it was a bit too async and stuff broke)

## v2.4.0 (2024-05-31)

### Features

* speed up `uvx list` (+ `uvx check`) with Futures and filtering before running checks (instead of after)
* speed up `upgrade-all`, `reinstall-all`, `upgrade-all` and allow filtering venv names

## v2.3.0 (2024-05-28)

### Feature

* `uvx check` to perform checks (like uvx list does) and report any problems.

### Refactoring

* improved Rust-esque codestyle (according to Clippy)

## v2.2.2 (2024-05-28)

### Fix

* `uvx upgrade` stored version metadata wrong

## v2.2.1 (2024-05-28)

### Fix

* `uvx list` was slow due to incorrect SSL behavior.

## v2.2.0 (2024-05-28)

### Features

* Added the `self` subcommand namespace
    * `uvx self update` to self-update
    * `uvx self changelog` to see the changelog of uvx
* Look for available updates on `uvx list`
    * Includes `--skip-updates`, `--show-prereleases`, `--ignore-constraints` as options

### BREAKING CHANGE

* `uvx self-update` is now `uvx self update`

## v2.1.0 (2024-05-15)

### Features

* Introduced the `uvx activate` command, enabling venv activation via bash function.
* Added `uvx setup`, allowing which handles installation of bash integration features (like `uvx activate` and tab
  completion).
* Added `uvx create` to create new (empty) virtualenvs without installing from a package.

### Fixes

* Enhanced shell compatibility by displaying a warning for unsupported shells during activation and hinting at the
  necessity of running `uvx setup` (and others).

### Documentation

* Provided detailed documentation for `uvx setup` to assist users in understanding its usage and configurations.

## v2.0.8 (2024-05-01)

### Docs

* extended description

## v2.0.7 (2024-05-01)

### Fix

* strip binary on release

## v2.0.6 (2024-04-26)

### Fix

* ensure `~/.local/bin` exists before trying to write symlinks

## v2.0.5 (2024-04-26)

### Updates

* **cargo**: bump dependencies

## v2.0.4 (2024-04-26)

### Fix

* **install**: show warnings if creating symlinks fails

## v2.0.3 (2024-04-26)

### Fix

* **self-update**: fall back to global Python if local (e.g. venv) one can not be found

## v2.0.2 (2024-04-26)

### Fix

* **self-update**: swap `before` and `after` version

## v2.0.1 (2024-04-26)

### Fix

* **.metadata**: add magic header so `file` understands it's binary data

## v2.0.0 (2024-04-26)

### BREAKING CHANGE

* Rewrite from Python to Rust.

## v1.x.x

See [CHANGELOG.md @ robinvandernoord/uvx](https://github.com/robinvandernoord/uvx/blob/master/CHANGELOG.md)
