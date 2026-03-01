# `bootiful-plymouth-discovery`

**Type:** Library crate (`lib`)  
**Path:** `crates/bootiful-plymouth-discovery/`  
**Purpose:** Core Plymouth system discovery logic. Produces a deterministic, complete digital-twin JSON snapshot of Plymouth state.

---

## Overview

This crate is the foundation of the Plymouth Discovery Layer. It collects all relevant system data and models it as strongly-typed Rust structs that serialize to a deterministic, 100% machine-readable JSON output.

It is **dependency-free from CLI and Tauri** — it only contains pure discovery logic and domain models.

---

## Public API

### `collect_all() -> Result<PlymouthConfig>`

The single entry point. Runs all 15 discovery phases sequentially and returns a fully populated `PlymouthConfig`.

```rust
use bootiful_plymouth_discovery::collect_all;

fn main() {
    let config = collect_all().expect("discovery failed");
    let json = serde_json::to_string_pretty(&config).unwrap();
    println!("{json}");
}
```

### `PlymouthConfig`

The top-level data model. All fields are serialized with `serde` and use `BTreeMap` for deterministic alphabetical key ordering.

```rust
pub struct PlymouthConfig {
    pub _meta:                Meta,
    pub runtime_environment:  RuntimeEnvironment,
    pub system_context:       SystemContext,
    pub packages:             Packages,
    pub global_config:        GlobalConfig,
    pub active_theme:         ActiveTheme,
    pub available_themes:     BTreeMap<String, AvailableTheme>,
    pub available_plugins:    BTreeMap<String, Plugin>,
    pub available_renderers:  BTreeMap<String, Renderer>,
    pub distribution_logo:    DistributionLogo,
    pub initramfs:            Initramfs,
    pub bootloader:           Bootloader,
    pub runtime_state:        RuntimeState,
    pub graphics:             Graphics,
    pub logs:                 Logs,
    pub consistency_check:    ConsistencyCheck,
}
```

---

## Discovery Phases

| Phase | Module | Description |
|:---:|---|---|
| 01 | `detector::environment` | UID, PID, PATH, DISPLAY, XDG |
| 02 | `detector::system` | uname, os-release, lsb_release, hostname |
| 03 | `detector::packages` | dpkg, apt-cache – all Plymouth packages |
| 04 | `detector::config` | plymouthd.conf/defaults, update-alternatives, symlink chains |
| 05 | `detector::themes` | Active theme: INI, plugin, ldd, SHA-256 asset hashes |
| 06 | `detector::themes` (scan) | All installed themes with full metadata |
| 07 | `detector::plugins` | `.so` plugin files with `ldd` dependency trees |
| 08 | `detector::plugins` (renderers) | DRM and frame-buffer renderer backends |
| 09 | `detector::logo` | OEM logo resolution (`special://logo`) |
| 10 | `detector::initramfs` | Boot image analysis via `lsinitramfs` |
| 11 | `detector::bootloader` | GRUB config, `/proc/cmdline`, kernel entries |
| 12 | `detector::runtime` | Daemon status, systemd units, display manager |
| 13 | `detector::graphics` | Framebuffer, DRM devices, lsmod, BGRT |
| 14 | `detector::logs` | Plymouth lines from syslog, journalctl, boot.log |
| 15 | `detector::consistency` | 7 automated pass/fail health checks |

---

## Active Theme Detection Chain

`plymouth-set-default-theme` does not exist on Ubuntu Noble / Linux Mint 22+. The crate uses this fallback chain:

1. `update-alternatives --query default.plymouth` → `Value:` line
2. Symlink resolution of `/etc/alternatives/default.plymouth`
3. Symlink resolution of `/usr/share/plymouth/themes/default.plymouth`
4. `Theme=` in `/etc/plymouth/plymouthd.conf`
5. `Theme=` in `/usr/share/plymouth/plymouthd.defaults`

---

## Consistency Checks

After all data is collected, 7 automated health checks are run:

| # | Check | Fix |
|:---:|---|---|
| 1 | `plymouth` package installed | `sudo apt install plymouth` |
| 2 | `plymouth-start.service` enabled/static | `sudo systemctl enable plymouth-start` |
| 3 | Active theme exists on disk | Re-install theme package |
| 4 | `splash` in `/proc/cmdline` | Add `splash` to `GRUB_CMDLINE_LINUX_DEFAULT` + `sudo update-grub` |
| 5 | Active theme present in initramfs | `sudo update-initramfs -u` |
| 6 | DRM/KMS modules in initramfs | `sudo update-initramfs -u` |
| 7 | Display manager depends on `plymouth-quit-wait` | Check DM unit `After=` |

---

## Cargo.toml Dependencies

```toml
[dependencies]
serde        = { workspace = true, features = ["derive"] }
serde_json   = { workspace = true }
chrono       = { workspace = true }
sha2         = { workspace = true }
walkdir      = { workspace = true }
glob         = { workspace = true }
regex        = { workspace = true }
libc         = { workspace = true }
thiserror    = { workspace = true }
anyhow       = { workspace = true }
```

---

## Error Handling

The crate uses `thiserror` for typed errors and `anyhow` for error propagation. Errors from individual detectors are gracefully handled — a single failing detector does not abort the entire collection.

```rust
pub enum Error {
    Io(#[from] std::io::Error),
    Json(#[from] serde_json::Error),
    CommandFailed(String),
    Parse(String),
}
```

---

## Permissions

Some data requires elevated privileges:

| Data | Privilege required |
|---|---|
| `lsinitramfs /boot/initrd.img-*` | root |
| `/var/log/syslog`, `/var/log/kern.log` | root on some systems |
| `journalctl -b` | root for all entries |

The crate runs without root — missing data is represented as `null` or empty arrays.

---

## Workspace Architecture

```
crates/
├── bootiful-plymouth-discovery/          ← this crate (core library)
│   └── src/
│       ├── lib.rs                        ← collect_all()
│       ├── model.rs                      ← PlymouthConfig + all structs
│       ├── error.rs                      ← Error types
│       └── detector/                    ← 15 detection modules
│
├── bootiful-plymouth-discovery-cli/      ← CLI binary (see its own doc)
└── bootiful-plymouth-discovery-tauri/   ← Tauri plugin (see its own doc)
```

---

## JSON Output Schema

All 16 top-level JSON sections produced by `collect_all()`:

> [!NOTE]
> All maps (`BTreeMap`) are serialized with **alphabetically sorted keys** for deterministic output.

### `_meta`

```json
{
  "generated_at": "2026-03-01T15:10:53",
  "generator": "bootiful-plymouth-discovery",
  "python_version": "3.12.3",
  "uid": 1000,
  "sections": ["runtime_environment", "system_context", ...]
}
```

| Key | Description |
|---|---|
| `generated_at` | ISO-8601 timestamp |
| `generator` | Crate name |
| `python_version` | Python version available on system |
| `uid` | UID of calling process (0 = root) |
| `sections` | Ordered list of all top-level sections |

### `runtime_environment`

| Key | Example | Description |
|---|---|---|
| `uid` | `0` | Unix user ID |
| `running_as_root` | `true` | Whether `euid == 0` |
| `pid` | `12345` | Process ID |
| `path` | `"/usr/local/sbin:..."` | `$PATH` |
| `display` | `":0"` | `$DISPLAY` |
| `xdg_session_type` | `"x11"` | `$XDG_SESSION_TYPE` |
| `xdg_current_desktop` | `"X-Cinnamon"` | `$XDG_CURRENT_DESKTOP` |

### `system_context`

| Key | Description |
|---|---|
| `uname_full` | Full `uname -a` output |
| `uname_machine` | `uname -m` (e.g. `x86_64`) |
| `dpkg_architecture` | `dpkg --print-architecture` |
| `hostname` | System hostname |
| `os_release` | All pairs from `/etc/os-release` |
| `lsb_release` | All pairs from `lsb_release -a` |

### `packages`

Each Plymouth package entry: `version`, `status`, `description` (omitted when null).

Special keys:
- `_plymouth_file_list` — all files from `dpkg -L plymouth`
- `_apt_cache_policy` — raw `apt-cache policy plymouth`
- `_apt_cache_show` — parsed fields: `Package`, `Version`, `Section`, `Origin`, `Maintainer`, `Installed-Size`

### `global_config`

- `etc_plymouth_directory` — `exists`, `files[]` with `path`, `size_bytes`, `modified`
- `daemon_defaults_file` / `daemon_conf_file` — `path`, `present`, `content`, `values.{Theme,ShowDelay,DeviceTimeout,CharacterEncoding}`
- `effective_daemon_config` — merged resolved values + `note`
- `active_theme` — resolved theme name string
- `update_alternatives_display` / `update_alternatives_query` — raw command output
- `symlink_chains` — per-link ordered array of `{ path, is_symlink, target?, exists }`

### `active_theme`

| Key | Description |
|---|---|
| `name` | Theme directory name (e.g. `"mint-logo"`) |
| `found` | Whether theme was located |
| `theme_dir` | Absolute path to theme directory |
| `plymouth_file` | Path to the `.plymouth` INI file |
| `plymouth_file_content` | Raw INI content |
| `plugin` | `ModuleName=` (e.g. `"two-step"`) |
| `script_file` | `ScriptFile=` or `null` |
| `image_dir` | `ImageDir=` or `null` |
| `extra_sections` | All non-`[Plymouth Theme]` INI sections |
| `plugin_so` | `{ path, size_bytes, ldd[] }` |
| `assets` | `[{ file, size_bytes, sha256_prefix }]` for all images |
| `script_file_detail` | *(omitted if null)* `{ path, exists, size_bytes, sha256 }` |

### `available_themes`

Map of `theme_name → { dir, plymouth_file, is_active, in_initramfs, name, description, plugin, script_file, image_dir, extra_sections, missing_files?, error? }`.

Themes on Linux Mint 22.3: `bgrt`, `details`, `mint-logo` (**active**), `mint-text`, `text`, `ubuntu-text`.

### `available_plugins`

Map of `plugin_name → { so_path, size_bytes, description, ldd[] }`.

Plugins: `details`, `label-pango`, `script`, `text`, `tribar`, `two-step`, `ubuntu-text`.

### `available_renderers`

Map of `renderer_name → { so_path, size_bytes, description }`.

| Renderer | Description |
|---|---|
| `drm` | **Preferred** – DRM/KMS via `/dev/dri/card0` |
| `frame-buffer` | **Fallback** – direct `/dev/fb0` |

### `distribution_logo`

`note` + map of `key → { path, exists, is_symlink?, resolved?, size_bytes?, description }`:

| Key | Path |
|---|---|
| `ubuntu_logo_png` | `/usr/share/plymouth/ubuntu-logo.png` |
| `mint_logo_png` | `/usr/share/pixmaps/mint-logo.png` |
| `distributor_logo_png` | `/usr/share/pixmaps/distributor-logo.png` |
| `distributor_logo` | `/etc/alternatives/distributor-logo` |

### `initramfs`

- `tools` — `update-initramfs`, `dracut`, `mkinitcpio` with `path` + `version`
- `current_kernel` — running kernel string
- `images[]` — `{ path, size_bytes, modified, is_current }`
- `current_initrd_analysis`:

| Key | Description |
|---|---|
| `path` / `exists` / `size_bytes` / `modified` | File metadata |
| `active_theme_modified` | Theme dir mtime |
| `theme_newer_than_initrd` | `true` → run `update-initramfs -u` |
| `plymouth_files` | `lsinitramfs` lines containing `"plymouth"` |
| `drm_kms_files` | DRM/KMS module paths in image |
| `bgrt_files` | BGRT paths in image |
| `active_theme_files` | Entries matching active theme name |
| `active_theme_in_initramfs` | Key health boolean |
| `error` | *(omitted if none)* |

> [!IMPORTANT]
> If `theme_newer_than_initrd` is `true` or `active_theme_in_initramfs` is `false`, run `sudo update-initramfs -u`.

### `bootloader`

- `type` — `"grub"` or `"systemd-boot"`
- `grub` — `present`, `default_config_path`, `cfg_path`, `cmdline_linux_default`, `cmdline_linux`, `non_comment_lines[]`, `splash_plymouth_entries_in_cfg[]`
- `systemd_boot` — `present`, `status`
- `active_cmdline` — `raw` + `parameters: { quiet, splash, nomodeset, plymouth.ignore-serial-consoles, vt.handoff }`
- `kernel_entries[]` — kernel version strings from `grub.cfg`

### `runtime_state`

| Key | Description |
|---|---|
| `version` | `plymouth --version` |
| `daemon_running` | `true` if `plymouth --ping` succeeds |
| `daemon_mode` | `boot`/`shutdown`/`suspend`/`resume`/`updates`/`system-upgrade`/`firmware-upgrade`/`"unknown"` |
| `valid_modes` | Reference list of all valid mode names |
| `mode_control_commands` | Commands for `plymouth change-mode` transitions |
| `units_list` | `systemctl list-units --all plymouth*` lines |
| `unit_states` | Per-unit `active` + `enabled` for 7 key services |
| `plymouth_quit_wait_properties` | `WantedBy`, `Before`, `After` |
| `plymouth_start_unit_file` | `systemctl cat plymouth-start.service` |
| `display_manager` | `service`, `active`, `depends_on_plymouth_quit_wait`, `after_property` |
| `run_plymouth_files` | Files in `/run/plymouth/` |

### `graphics`

- `framebuffer_devices[]` — `{ device, virtual_size, bits_per_pixel, stride }`
- `graphics_sys_entries[]` — entries from `/sys/class/graphics/*`
- `drm_devices[]` — `{ name, enabled, driver }` from `/sys/class/drm/card*`
- `loaded_graphics_modules[]` — graphics modules from `lsmod`
- `simpledrm_loaded`, `efifb_loaded`, `uefi` — booleans
- `bgrt` — `{ present, xoffset?, yoffset?, status?, type? }`

> [!NOTE]
> If `bgrt.present` is `false`, the `bgrt` Plymouth theme falls back to its `bgrt-fallback.png` asset.

### `logs`

- `boot_log` — `{ path, exists, plymouth_lines[] }`
- `syslog` — `{ path, exists, plymouth_lines_last_20[] }`
- `kern_log` — `{ path, exists, plymouth_lines_last_10[] }`
- `journalctl_current_boot` — `{ plymouth_lines_last_40[] }`

### `consistency_check`

```json
{
  "all_ok": true,
  "passed": ["Package 'plymouth' installed (ver 24.004.60-1ubuntu7)", ...],
  "issues": []
}
```

| # | Check | Pass condition |
|:---:|---|---|
| 1 | Package installed | `dpkg-query` returns a version |
| 2 | Unit enabled | `plymouth-start.service` is `enabled` or `static` |
| 3 | Theme on disk | Theme name exists in theme directories |
| 4 | `splash` in cmdline | `/proc/cmdline` contains `splash` |
| 5 | Theme in initramfs | `lsinitramfs` output contains theme name |
| 6 | DRM in initramfs | `lsinitramfs` output contains DRM/KMS paths |
| 7 | DM integration | Display manager `After=` includes `plymouth` |
