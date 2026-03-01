use crate::model::{GlobalConfig, DirectoryInfo, FileInfo, ConfigFile};
use crate::detector::utils::{run, read_file, file_exists, file_size, file_modified, symlink_chain};
use std::collections::BTreeMap;
use std::path::Path;
use regex::Regex;

pub fn collect() -> GlobalConfig {
    let etc_plymouth = "/etc/plymouth";
    let mut etc_files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(etc_plymouth) {
        let mut paths: Vec<_> = entries.filter_map(|e| e.ok()).map(|e| e.path()).collect();
        paths.sort();
        for path in paths {
            etc_files.push(FileInfo {
                path: path.to_string_lossy().into_owned(),
                size_bytes: file_size(&path),
                modified: file_modified(&path),
            });
        }
    }

    let defaults_path = "/usr/share/plymouth/plymouthd.defaults";
    let conf_path = "/etc/plymouth/plymouthd.conf";

    let defaults_file = collect_config_file(defaults_path);
    let conf_file = collect_config_file(conf_path);

    let active_theme = detect_active_theme(&conf_file, &defaults_file);

    let mut effective = BTreeMap::new();
    effective.insert("note".to_string(), serde_json::Value::String("Resolved: conf overrides defaults; theme via update-alternatives when no conf".to_string()));
    
    let keys = ["Theme", "ShowDelay", "DeviceTimeout", "CharacterEncoding"];
    for key in keys {
        let val = conf_file.values.get(key)
            .or_else(|| defaults_file.values.get(key))
            .cloned()
            .flatten();
        
        effective.insert(key.to_string(), match val {
            Some(v) => serde_json::Value::String(v),
            None => {
                if key == "Theme" {
                    serde_json::Value::String(active_theme.clone())
                } else if key == "ShowDelay" {
                    serde_json::Value::Number(0.into())
                } else if key == "DeviceTimeout" {
                    serde_json::Value::Number(8.into())
                } else {
                    serde_json::Value::Null
                }
            }
        });
    }

    let mut symlink_chains = BTreeMap::new();
    for link in ["/etc/alternatives/default.plymouth", "/usr/share/plymouth/themes/default.plymouth"] {
        if file_exists(link) || std::fs::read_link(link).is_ok() {
            symlink_chains.insert(link.to_string(), symlink_chain(link));
        }
    }

    GlobalConfig {
        etc_plymouth_directory: DirectoryInfo {
            exists: Path::new(etc_plymouth).exists(),
            files: etc_files,
        },
        daemon_defaults_file: defaults_file,
        daemon_conf_file: conf_file,
        effective_daemon_config: effective,
        active_theme: active_theme.clone(),
        update_alternatives_display: run("update-alternatives", &["--display", "default.plymouth"]),
        update_alternatives_query: run("update-alternatives", &["--query", "default.plymouth"]),
        symlink_chains,
    }
}

fn collect_config_file(path: &str) -> ConfigFile {
    let content = if file_exists(path) { Some(read_file(path)) } else { None };
    let mut values = BTreeMap::new();
    let keys = ["Theme", "ShowDelay", "DeviceTimeout", "CharacterEncoding"];
    
    if let Some(ref txt) = content {
        for key in keys {
            let re = Regex::new(&format!(r"^{}\s*=\s*(.+)$", key)).unwrap();
            let val = txt.lines()
                .find_map(|l| re.captures(l).map(|c| c[1].trim().to_string()));
            values.insert(key.to_string(), val);
        }
    } else {
        for key in keys {
            values.insert(key.to_string(), None);
        }
    }

    ConfigFile {
        path: path.to_string(),
        present: file_exists(path),
        content,
        values,
    }
}

fn detect_active_theme(conf: &ConfigFile, defaults: &ConfigFile) -> String {
    // 1. update-alternatives
    let query = run("update-alternatives", &["--query", "default.plymouth"]);
    for line in query.lines() {
        if line.starts_with("Value:") {
            if let Some(path) = line.splitn(2, ':').nth(1) {
                let p = Path::new(path.trim());
                if p.extension().map(|e| e == "plymouth").unwrap_or(false) {
                    if let Some(name) = p.parent().and_then(|parent| parent.file_name()) {
                        return name.to_string_lossy().into_owned();
                    }
                }
            }
        }
    }

    // 2. symlink chains
    for link in ["/etc/alternatives/default.plymouth", "/usr/share/plymouth/themes/default.plymouth"] {
        let chain = symlink_chain(link);
        if let Some(last) = chain.last() {
            let p = Path::new(&last.path);
            if p.extension().map(|e| e == "plymouth").unwrap_or(false) {
                if let Some(name) = p.parent().and_then(|parent| parent.file_name()) {
                    return name.to_string_lossy().into_owned();
                }
            }
        }
    }

    // 3. config files
    conf.values.get("Theme").cloned().flatten()
        .or_else(|| defaults.values.get("Theme").cloned().flatten())
        .unwrap_or_default()
}
