# Bootiful

Bootiful is a graphical tool to preview, edit, and safely manage Plymouth boot themes.

## Plymouth Discovery Layer

The Plymouth Discovery Layer is a Rust Cargo workspace providing complete, deterministic system snapshots of Plymouth configuration.

See [`docs/plymouth_detect.md`](docs/plymouth_detect.md) for the architecture overview, or jump directly to a crate:

- [`docs/crate-bootiful-plymouth-discovery.md`](docs/crate-bootiful-plymouth-discovery.md) – Core library
- [`docs/crate-bootiful-plymouth-discovery-cli.md`](docs/crate-bootiful-plymouth-discovery-cli.md) – CLI tool
- [`docs/crate-bootiful-plymouth-discovery-tauri.md`](docs/crate-bootiful-plymouth-discovery-tauri.md) – Tauri plugin

### Quick Start

```bash
cargo run --package bootiful-plymouth-discovery-cli -- --output discovery.json
```
