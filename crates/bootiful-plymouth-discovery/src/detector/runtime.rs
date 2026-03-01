use crate::model::{RuntimeState, UnitState, DisplayManagerInfo, RunPlymouthFile};
use crate::detector::utils::{run, run_with_status, read_file, file_exists, file_size};
use std::collections::BTreeMap;

pub fn collect() -> RuntimeState {
    let version = run("plymouth", &["--version"]);
    let (ping_rc, _, _) = run_with_status("plymouth", &["--ping"]);
    let is_running = ping_rc == 0;

    let daemon_mode = if is_running { run("plymouth", &["--get-mode"]) } else { "unknown".into() };

    let units_out = run("systemctl", &["list-units", "--all", "--no-pager", "plymouth*", "--no-legend", "--output=short"]);
    let units_list = units_out.lines().map(|s| s.trim().to_string()).collect();

    let key_units = [
        "plymouth-start.service", "plymouth-quit.service", "plymouth-quit-wait.service",
        "plymouth-read-write.service", "plymouth-halt.service", "plymouth-poweroff.service", "plymouth-reboot.service"
    ];
    let mut unit_states = BTreeMap::new();
    for u in key_units {
        unit_states.insert(u.to_string(), UnitState {
            active: run("systemctl", &["is-active", u]),
            enabled: run("systemctl", &["is-enabled", u]),
        });
    }

    let mut show_props = BTreeMap::new();
    let show_out = run("systemctl", &["show", "plymouth-quit-wait.service", "--property=WantedBy,RequiredBy,Before,After"]);
    for line in show_out.lines() {
        if let Some((k, v)) = line.split_once('=') {
            show_props.insert(k.to_string(), v.to_string());
        }
    }

    let cat_out = run("systemctl", &["cat", "plymouth-start.service"]);

    let mut dm_info = DisplayManagerInfo { service: None, active: false, depends_on_plymouth_quit_wait: false, after_property: None };
    for dm in ["gdm.service", "gdm3.service", "lightdm.service", "sddm.service"] {
        if run("systemctl", &["is-active", dm]) == "active" {
            let deps = run("systemctl", &["show", dm, "--property=After"]);
            dm_info = DisplayManagerInfo {
                service: Some(dm.to_string()),
                active: true,
                depends_on_plymouth_quit_wait: deps.to_lowercase().contains("plymouth"),
                after_property: Some(deps),
            };
            break;
        }
    }

    let mut run_plymouth_files = Vec::new();
    let run_plymouth = "/run/plymouth";
    if file_exists(run_plymouth) {
        if let Ok(entries) = std::fs::read_dir(run_plymouth) {
            let mut paths: Vec<_> = entries.filter_map(|e| e.ok()).map(|e| e.path()).collect();
            paths.sort();
            for path in paths {
                run_plymouth_files.push(RunPlymouthFile {
                    path: path.to_string_lossy().into_owned(),
                    size_bytes: file_size(&path),
                    content: if path.is_file() { Some(read_file(&path)) } else { None },
                });
            }
        }
    }

    let mut mode_control_commands = BTreeMap::new();
    mode_control_commands.insert("quit_retain_splash".into(), "plymouth quit --retain-splash".into());
    mode_control_commands.insert("shutdown".into(), "plymouth change-mode --shutdown".into());
    mode_control_commands.insert("suspend".into(), "plymouth change-mode --suspend".into());
    mode_control_commands.insert("resume".into(), "plymouth change-mode --resume".into());
    mode_control_commands.insert("updates".into(), "plymouth change-mode --updates".into());
    mode_control_commands.insert("system_upgrade".into(), "plymouth change-mode --system-upgrade".into());

    RuntimeState {
        version: if version.is_empty() { None } else { Some(version) },
        daemon_running: is_running,
        daemon_mode,
        valid_modes: vec!["boot".into(), "shutdown".into(), "suspend".into(), "resume".into(), "updates".into(), "system-upgrade".into(), "firmware-upgrade".into()],
        mode_control_commands,
        units_list,
        unit_states,
        plymouth_quit_wait_properties: show_props,
        plymouth_start_unit_file: if cat_out.is_empty() { None } else { Some(cat_out) },
        display_manager: dm_info,
        run_plymouth_files,
    }
}
