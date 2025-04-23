# uvenv Snap Package

uvenv is a simple command-line tool for managing virtual environments, written in Rust. Think of it as pipx, but for uv.

## Getting started

  ```bash
  snap install uvenv
  ```

## Snap Installation Caveats

When installed via Snap, there are some important differences to note:

- Tools are downloaded to `~/snap/uvenv/current/.local/uvenv` instead of `~/.local/uvenv`
- Scripts are installed in `~/snap/uvenv/current/.local/bin` instead of `~/.local/bin`
- The snap package cannot update files like `~/.bashrc` or perform self-updates.

### Thawing with Snap
Note that due to snap's strict sandboxing, `uvenv thaw` can not access `uvenv.lock` in most directories 
and the command will fail with a permission error.
Instead, it must be run from the snap directory:
```bash
cd ~/snap/uvenv/current
ls
# uvenv.lock
uvenv thaw
```

## Setting Up Bash Integration

To enable all Bash-specific features, add the following lines to your `~/.bashrc`:

```bash
eval "$(uvenv --generate=bash ensurepath)" # Fix PATH (or you can add `~/snap/uvenv/current/.local/bin` to your PATH manually)
eval "$(uvenv --generate=bash completions)" # Optional: Enable tab completion
eval "$(uvenv --generate=bash activate _)" # Optional: Enable the `uvenv activate` command
```

For other shells, run to display the appropriate setup instructions:

```bash
uvenv setup
```


