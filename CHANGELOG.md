# Changelog

<!--next-version-placeholder-->

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
