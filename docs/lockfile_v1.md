# Lockfile V1 Format

The `uvenv.lock` file is a TOML-based lockfile (with support for other output formats)
that pins exact package versions and configurations for reproducible environments.

## Structure

```toml
version = 1

[packages]
# Package specifications go here
```

### Package Specification

Each package in the `[packages]` table supports the following options:

| Field      | Type   | Required | Description                                                |
|------------|--------|----------|------------------------------------------------------------|
| `spec`     | string | Yes      | Package specifier (name, with optional constraints)        |
| `version`  | string | No       | Pinned version number                                      |
| `python`   | string | No       | Exact Python version the package was locked for (or empty) |
| `injected` | array  | No       | List of injected dependency names                          |
| `editable` | bool   | No       | Whether the package is installed in editable mode          |

## Formats

### Minimal (Compact Format)

```toml
version = 1

[packages]
python-semantic-release = { spec = "python-semantic-release<8", version = "<8" }
pgcli = { spec = "pgcli", version = "~=4.3.0", python = "3.13", injected = ["psycopg-binary", "psycopg2-binary"] }
```

### Compact (Official Output)

```toml
version = 1

[packages]
python-semantic-release = { spec = "python-semantic-release<8", version = "<8", python = "3.14", injected = [], editable = false }
pgcli = { spec = "pgcli", version = "~=4.3.0", python = "3.13", injected = ["psycopg-binary", "psycopg2-binary"], editable = false }
```

### Expanded (Alternative)

```toml
version = 1

[packages.python-semantic-release]
spec = "python-semantic-release<8"
version = "<8"
python = "3.14"
injected = []
editable = false

[packages.pgcli]
spec = "pgcli"
version = "~=4.3.0"
python = "3.13"
injected = ["psycopg-binary", "psycopg2-binary"]
editable = false
```
