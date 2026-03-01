# `bootiful-plymouth-discovery-cli`

**Type:** Binary crate  
**Path:** `crates/bootiful-plymouth-discovery-cli/`  
**Purpose:** Command-line interface for the Plymouth Discovery Layer. Generates a full JSON snapshot of Plymouth state to stdout or a file.

---

## Overview

This crate is a thin, ergonomic CLI wrapper around the `bootiful-plymouth-discovery` core library. It is the primary way to run the discovery tool from a terminal.

---

## Usage

```bash
# From the workspace root:
cargo run --package bootiful-plymouth-discovery-cli -- --output discovery.json

# With custom indentation:
cargo run --package bootiful-plymouth-discovery-cli -- --output discovery.json --indent 4

# Print to stdout:
cargo run --package bootiful-plymouth-discovery-cli

# After building a release binary:
./target/release/bootiful-plymouth-discovery-cli --output discovery.json
```

### Arguments

| Argument | Type | Default | Description |
|---|---|---|---|
| `--output`, `-o` | `PATH` | stdout | Path to write JSON output |
| `--indent` | `usize` | `2` | JSON indentation spaces |

---

## Build

```bash
# Development build:
cargo build --package bootiful-plymouth-discovery-cli

# Optimized release build:
cargo build --release --package bootiful-plymouth-discovery-cli
```

The release binary is placed at `target/release/bootiful-plymouth-discovery-cli`.

---

## Permissions

For complete output (initramfs analysis, system logs), run with elevated privileges:

```bash
sudo ./target/release/bootiful-plymouth-discovery-cli --output discovery.json
```

Without root, the tool still runs and produces a valid JSON — but `initramfs`, `logs`, and some `packages` fields will be empty or null.

---

## Output

A single JSON file with 16 top-level sections. Typical size:

| Mode | Size |
|---|---|
| Without root | ~65 KB |
| With root (complete) | ~75–80 KB |

Example (abbreviated):

```json
{
  "_meta": {
    "generated_at": "2026-03-01T15:10:53",
    "generator": "bootiful-plymouth-discovery",
    "uid": 0,
    "sections": ["runtime_environment", "system_context", ...]
  },
  "active_theme": {
    "name": "mint-logo",
    "found": true,
    "plugin": "two-step",
    ...
  },
  "consistency_check": {
    "all_ok": true,
    "passed": ["Package 'plymouth' installed (ver 24.004.60-1ubuntu7)", ...],
    "issues": []
  }
}
```

For the complete JSON schema see [`crate-bootiful-plymouth-discovery.md`](./crate-bootiful-plymouth-discovery.md#json-output-schema).

---

## Cargo.toml Dependencies

```toml
[dependencies]
bootiful-plymouth-discovery = { path = "../bootiful-plymouth-discovery" }
clap                        = { workspace = true, features = ["derive"] }
serde_json                  = { workspace = true }
anyhow                      = { workspace = true }
```
