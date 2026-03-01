use crate::model::{Bootloader, GrubConfig, SystemdBootConfig, ActiveCmdline};
use crate::detector::utils::{run, read_file, file_exists};
use std::collections::BTreeMap;
use regex::Regex;

pub fn collect() -> Bootloader {
    let grub_default = "/etc/default/grub";
    let grub_cfg = "/boot/grub/grub.cfg";
    let grub_txt = read_file(grub_default);
    let proc_cmdline = read_file("/proc/cmdline");

    let params_list = ["quiet", "splash", "nomodeset", "plymouth.ignore-serial-consoles", "vt.handoff"];
    let mut parameters = BTreeMap::new();
    for p in params_list {
        parameters.insert(p.to_string(), proc_cmdline.contains(p));
    }

    let cmdline_linux_default = match Regex::new(r#"^GRUB_CMDLINE_LINUX_DEFAULT="([^"]*)""#).unwrap().captures(&grub_txt) {
        Some(c) => c[1].to_string(),
        None => String::new(),
    };
    let cmdline_linux = match Regex::new(r#"^GRUB_CMDLINE_LINUX="([^"]*)""#).unwrap().captures(&grub_txt) {
        Some(c) => c[1].to_string(),
        None => String::new(),
    };

    let mut kernel_entries = Vec::new();
    if file_exists(grub_cfg) {
        let content = read_file(grub_cfg);
        let re = Regex::new(r"vmlinuz-(\S+)").unwrap();
        for cap in re.captures_iter(&content) {
            let ver = cap[1].to_string();
            if !kernel_entries.contains(&ver) {
                kernel_entries.push(ver);
            }
        }
    }

    let splash_plymouth_entries = if file_exists(grub_cfg) {
        read_file(grub_cfg).lines()
            .filter(|l| l.contains("splash") || l.to_lowercase().contains("plymouth"))
            .map(|s| s.trim().to_string())
            .take(30)
            .collect()
    } else {
        Vec::new()
    };

    let sdboot = file_exists("/usr/bin/bootctl");
    let sdboot_status = if sdboot { Some(run("bootctl", &["status"])) } else { None };

    Bootloader {
        bootloader_type: if sdboot && !file_exists(grub_cfg) { "systemd-boot".into() } else { "grub".into() },
        grub: GrubConfig {
            present: file_exists(grub_cfg),
            default_config_path: if file_exists(grub_default) { Some(grub_default.into()) } else { None },
            cfg_path: if file_exists(grub_cfg) { Some(grub_cfg.into()) } else { None },
            cmdline_linux_default,
            cmdline_linux,
            non_comment_lines: grub_txt.lines()
                .filter(|l| !l.trim().is_empty() && !l.trim().starts_with('#'))
                .map(|s| s.to_string())
                .collect(),
            splash_plymouth_entries_in_cfg: splash_plymouth_entries,
        },
        systemd_boot: SystemdBootConfig {
            present: sdboot,
            status: sdboot_status,
        },
        active_cmdline: ActiveCmdline {
            raw: proc_cmdline,
            parameters,
        },
        kernel_entries,
    }
}
