# Changelog

<!--next-version-placeholder-->

## 3.10.14 (2025-12-03)

### Updates

* update uv dependencies to 0.9.15

## 3.10.13 (2025-11-26)

### Updates

* update uv dependencies to 0.9.12

## 3.10.12 (2025-11-21)

### Updates

* update uv dependencies to 0.9.11

### Documentation

* explained lockfile (v1) specification

## 3.10.11 (2025-11-13)

### Updates

* update uv dependencies to 0.9.9

## 3.10.10 (2025-10-30)

### Updates

* update uv dependencies to 0.9.6
* update Rust version to 1.91

## 3.10.9 (2025-10-29)

### Fix

* allow `uvenv install -e .` again

## 3.10.8 (2025-10-22)

### Updates

* update uv dependencies to 0.9.5

## 3.10.7 (2025-10-16)

### Updates

* update uv dependencies to 0.9.3

## 3.10.6 (2025-10-08)

### Updates

* update uv dependencies to 0.9.0

## 3.10.5 (2025-09-24)

### Updates

* update uv dependencies to 0.8.22

## 3.10.4 (2025-09-18)

### Updates

* update uv dependencies to 0.8.18

## 3.10.3 (2025-09-03)

### Updates

* update uv dependencies to 0.8.15

## 3.10.2 (2025-08-29)

### Updates

* update uv dependencies to 0.8.14

## 3.10.1 (2025-08-19)

### Updates

* update uv dependencies to 0.8.12

## 3.10.0 (2025-08-12)

### Deprecations

**macOS (x64) build removed**  
Due to decreasing support for Intel-based (x64) macOS builds from GitHub and the Rust project, the `x86_64-apple-darwin` target has been removed from our build pipeline.

- [Rust announcement](https://blog.rust-lang.org/2025/08/07/Rust-1.89.0/#demoting-x86-64-apple-darwin-to-tier-2-with-host-tools)
- [Rust RFC 3841](https://github.com/rust-lang/rfcs/pull/3841)
- [GitHub macOS runner changes](https://github.blog/changelog/2025-07-11-upcoming-changes-to-macos-hosted-runners-macos-latest-migration-and-xcode-support-policy-updates/#macos-13-is-closing-down)


## 3.9.20 (2025-08-12)

### Updates

* update uv dependencies to 0.8.9

## 3.9.19 (2025-08-07)

### Updates

* update uv dependencies to 0.8.5

## 3.9.18 (2025-07-31)

### Updates

* update uv dependencies to 0.8.4

## 3.9.17 (2025-07-25)

### Updates

* update uv dependencies to 0.8.3

## 3.9.16 (2025-07-19)

## Fixes (snap-release)

* make uv install python versions in a common (instead of revision-specific) snap folder
  by default, uv in snap would install at
  `~/snap/uvenv/<revision>/.local/share/uv/python/`
  meaning it would be moved after each update;
  leading to longer update times and breaking symlinks.
  so, we set the install dir to a fixed location (`~/snap/uvenv/common/python`)

### Updates

* update other dependencies (toml)

## 3.9.15 (2025-07-18)

### Updates

* update uv dependencies to 0.8.0

## 3.9.14 (2025-07-10)

### Updates

* update uv dependencies to 0.7.20

## 3.9.13 (2025-06-30)

### Updates

* update uv dependencies to 0.7.17

## 3.9.12 (2025-06-24)

### Updates

* update uv dependencies to 0.7.14

## 3.9.11 (2025-06-13)

### Fixes

* properly store requested `--python` install so reinstall won't switch python versions
* proper semantic version comparision so 3.10 > 3.9

## 3.9.10 (2025-06-13)

### Fixes

* on snap, store everything in the 'common' (`$SNAP_USER_COMMON`)
  instead of `$SNAP_USER_DATA`  
  (snap revision-specific 'home')

## 3.9.9 (2025-06-13)

### Updates

* update uv dependencies to 0.7.13

## 3.9.8 (2025-06-06)

Addresses issues from [#17](https://github.com/robinvandernoord/uvenv/issues/17).

### Fixes

* change directory when running `uv` commands to prevent reading local `pyproject.toml` config, instead of setting
  UV_NO_CONFIG.

### Documentation

* added `uv tool install uvenv` as a proper installation method for users who already have `uv`. Fixed structure for
  other methods (description, advantages, considerations, installation steps)
* added a section on the difference with `uv tool`

## 3.9.7 (2025-06-04)

### Updates

* update uv dependencies to 0.7.10

## 3.9.6 (2025-05-30)

### Updates

* update uv dependencies to 0.7.8

## 3.9.5 (2025-05-23)

### Fixes

* use `UV_NO_CONFIG=1` instead of `--no-config` for every `uv` command

## 3.9.4 (2025-05-23)

### Minor Enhancements

* add `update` as alias for `upgrade`

### Updates

* update uv dependencies to 0.7.7

## 3.9.3 (2025-05-21)

### Updates

* update uv dependencies to 0.7.6

## 3.9.2 (2025-05-08)

### Minor Enhancements

* nicer display for `self info`. Defaults to `fancy` with options for `basic`, `toml` and `json`
* github action: split wheel builds into one job per target for better parallelism

## 3.9.1 (2025-05-08)

### Fixes

* pass `--no-config` to `uv` to prevent looking at local `pyproject.toml` files

### Snapcraft

* use $SNAP (`~/snap/uvenv/<revision>`) as workdir
  (no need using `.local/uvenv` if the files are already scoped to uvenv)

### Updates

* bump to uv 0.7.3

## 3.9.0 (2025-04-30)

### Features

* `uvenv upgrade` now supports a dynamic amount of packages:
    - `uvenv upgrade <package>` like before
    - `uvenv upgrade` will upgrade all outdated packages
    - `uvenv upgrade <package1> <package2> ...` to upgrade multiple

    + use `uvenv upgrade-all` to upgrade all packages without checking for oudated

### Updates

* bump to uv 0.7.0

## 3.8.4 (2025-04-23)

### Fixes

* use atomic file writes to prevent metadata disappearing

## 3.8.3 (2025-04-23)

### Fixes

* sort alphabetically on `uvenv list`
    + pt2 improved metadata check for (self) install via installation script (ignore instead of fill with empty
      metadata -> `remove_all` won't self-remove)

## 3.8.2 (2025-04-23)

### Fixes

* improved metadata check for self-install via installation script (`install.sh`)
    * fixes `Metadata for 'uvenv' could not be loaded.`

### Updates

* bump to uv 0.6.16

### Docs

* Add install script and recommend it if global `pip` install isn't possible (e.g. Ubuntu 24.04)  in docs

## 3.8.1 (2025-04-17)

### Fix

* include `LockfileV0` in outside of `debug_assertions` so `--release` properly builds again.

## 3.8.0 (2025-04-17)

### Features

* Added `uvenv freeze` command to generate a lockfile of installed applications, with support for custom filenames,
  formats (`json`, `toml`, `binary`), and inclusion/exclusion filters.
* Added `uvenv thaw` command to reinstall applications from a lockfile, with options to remove or skip existing
  environments, filter specific dependencies, and control Python version resolution.

## 3.7.5 (2025-04-14)

### Updates

* bump to uv 0.6.14
* bump Tokio due to CWE-664

## 3.7.4 (2025-04-08)

### Updates

* bump to uv 0.6.13

## 3.7.3 (2025-03-31)

### Updates

* bump to uv 0.6.11

## 3.7.2 (2025-03-26)

### Updates

* bump to uv 0.6.10

## 3.7.1 (2025-03-14)

### Fixes

* add aliases like `ls`, `rm` for lazy people

### Documentation

* explained more about the snap version, with caveats and tips

## 3.7.0 (2025-03-13)

### Features

* replace `self version` with more verbose `self info`
* First release of `uvenv` on `snap` - `snap install uvenv`,
  with some features (self-update, rcfile editing) disabled because of snapcraft's strict permission system.

## 3.6.5 (2025-03-12)

### Build

* Better darwin support (don't suggest installing uvx 1.0)
* Change build script so `macos` can cross-compile all wheels

## 3.6.4 (2025-03-12)

### Updates

* bump to uv 0.6.6

### Internals

* use Rust 2024's async closures to create `run_if_supported_shell_else_warn_async`

## 3.6.3 (2025-02-25)

### Updates

* bump to uv 0.6.3
* bump to rust 2024 edition (1.85)

## 3.6.2 (2025-02-14)

### Updates

* bump to uv 0.6.0

## 3.6.1 (2025-02-05)

### Fix

* improve script detection for symlinks by using uv's logic
    * This means the binary `task` will be found when installing `go-task-bin`

### Updates

* upgrade to `uv` 0.5.28

## 3.6.0 (2025-01-29)

### Feature

* extend functionality of `uvenv check` to check interpreter of shebang scripts

### Updates

* upgrade to `uv` 0.5.25

## 3.5.3 (2025-01-11)

### Fix

* fix(check): look at actually installed version via freeze instead of cached installed version from metadata
    + relevant when package is updated outside of uvenv (manual pip update or package self-update)

### Updates

* upgrade to `uv` 0.5.17

## 3.5.2 (2025-01-03)

### Updates

* upgrade to `uv` 0.5.14

## 3.5.1 (2024-12-04)

### Updates

* upgrade to `uv` 0.5.6

## 3.5.0 (2024-11-13)

### Features

* Add support for macOS and zsh

## 3.4.6 (2024-11-10)

### Updates

* upgrade to `uv` 0.5.1

## 3.4.5 (2024-11-08)

### Updates

* upgrade to `uv` 0.5.0

## 3.4.4 (2024-11-05)

### Updates

* upgrade to `uv` 0.4.30

## 3.4.3 (2024-10-17)

### Updates

* upgrade to `uv` 0.4.23

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
