use crate::model::ConsistencyCheck;
use crate::detector::utils::run_with_status;
use crate::model::PlymouthConfig;

pub fn check(config: &PlymouthConfig) -> ConsistencyCheck {
    let mut passed = Vec::new();
    let mut issues = Vec::new();

    // 1. Package installed
    let (rc, ver, _) = run_with_status("dpkg-query", &["-W", "-f=${Version}", "plymouth"]);
    if rc == 0 && !ver.is_empty() {
        passed.push(format!("Package 'plymouth' installed (ver {})", ver));
    } else {
        issues.push("Package 'plymouth' is NOT installed".to_string());
    }

    // 2. plymouth-start.service enabled
    let start_enabled = config.runtime_state.unit_states.get("plymouth-start.service")
        .map(|s| s.enabled.as_str()).unwrap_or("unknown");
    if start_enabled == "enabled" || start_enabled == "static" {
        passed.push(format!("plymouth-start.service: {}", start_enabled));
    } else {
        issues.push(format!("plymouth-start.service not enabled (state: {})", start_enabled));
    }

    // 3. Active theme on disk
    let active = &config.global_config.active_theme;
    if !active.is_empty() && config.available_themes.contains_key(active) {
        passed.push(format!("Active theme '{}' exists on disk", active));
    } else {
        issues.push(format!("Active theme '{}' NOT found in theme directories", active));
    }

    // 4. splash in cmdline
    if config.bootloader.active_cmdline.parameters.get("splash") == Some(&true) {
        passed.push("Kernel parameter 'splash' present in active cmdline".to_string());
    } else {
        issues.push("Kernel parameter 'splash' missing from active cmdline".to_string());
    }

    // 5. Theme in initramfs
    let analysis = &config.initramfs.current_initrd_analysis;
    if analysis.active_theme_in_initramfs {
        passed.push(format!("Active theme '{}' present in initramfs", active));
    } else {
        issues.push(format!("Active theme '{}' NOT in initramfs – run: update-initramfs -u", active));
    }

    // 6. DRM in initramfs
    if !analysis.drm_kms_files.is_empty() {
        passed.push("DRM/KMS modules present in initramfs".to_string());
    } else {
        issues.push("DRM/KMS modules missing from initramfs".to_string());
    }

    // 7. Display manager integration
    let dm = &config.runtime_state.display_manager;
    if dm.active && dm.depends_on_plymouth_quit_wait {
        if let Some(ref svc) = dm.service {
            passed.push(format!("{} properly depends on plymouth-quit-wait.service", svc));
        }
    } else if dm.active && !dm.depends_on_plymouth_quit_wait {
        if let Some(ref svc) = dm.service {
            issues.push(format!("{} does NOT depend on plymouth-quit-wait", svc));
        }
    }

    ConsistencyCheck {
        all_ok: issues.is_empty(),
        passed,
        issues,
    }
}
