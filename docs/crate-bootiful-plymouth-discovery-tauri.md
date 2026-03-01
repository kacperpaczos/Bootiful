# `bootiful-plymouth-discovery-tauri`

**Type:** Library crate (`lib`)  
**Path:** `crates/bootiful-plymouth-discovery-tauri/`  
**Purpose:** Tauri plugin adapter that exposes the Plymouth Discovery Layer as an invokable Tauri command.

---

## Overview

This crate bridges the `bootiful-plymouth-discovery` core library with any Tauri application. It registers a Tauri plugin with a single command that frontend code (e.g. TypeScript/React) can call via `invoke`.

---

## Integration

### 1. Add to Tauri app `Cargo.toml`

```toml
[dependencies]
bootiful-plymouth-discovery-tauri = { path = "../crates/bootiful-plymouth-discovery-tauri" }
```

### 2. Register the plugin in `main.rs`

```rust
fn main() {
    tauri::Builder::default()
        .plugin(bootiful_plymouth_discovery_tauri::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 3. Invoke from the frontend

```typescript
import { invoke } from "@tauri-apps/api/core";

const config = await invoke("get_plymouth_config");
console.log(config.active_theme.name);  // e.g. "mint-logo"
console.log(config.consistency_check.all_ok);  // true/false
```

---

## Exported Tauri Command

### `get_plymouth_config`

Runs all 15 discovery phases and returns the complete `PlymouthConfig` object as a JSON-serializable Tauri response.

```rust
#[tauri::command]
pub fn get_plymouth_config<R: Runtime>() -> Result<PlymouthConfig, String>
```

| Behavior | Detail |
|---|---|
| Success | Returns the full `PlymouthConfig` struct (serialized as JSON by Tauri) |
| Error | Returns an error string describing what failed |
| Permissions | Same as the CLI — root recommended for complete data |

---

## Plugin Registration

```rust
pub fn init<R: Runtime>() -> tauri::plugin::TauriPlugin<R> {
    tauri::plugin::Builder::new("plymouth-discovery")
        .invoke_handler(tauri::generate_handler![get_plymouth_config])
        .build()
}
```

The plugin name is `"plymouth-discovery"` — this is the namespace used internally by Tauri.

---

## Cargo.toml Dependencies

```toml
[dependencies]
bootiful-plymouth-discovery = { path = "../bootiful-plymouth-discovery" }
tauri                       = { workspace = true }
```

---

## Notes

> [!NOTE]
> This crate does **not** include a Tauri `main.rs` or `tauri.conf.json` — it is a plugin to be embedded into an existing Tauri application.

> [!IMPORTANT]
> Because `plymouthctl` and `lsinitramfs` require root privileges, the Tauri app must either run as root or configure `allowlist` + privilege escalation for full JSON output.
