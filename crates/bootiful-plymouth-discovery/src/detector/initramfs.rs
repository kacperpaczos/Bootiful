use crate::model::{Initramfs, ToolInfo, InitrdImage, InitrdAnalysis};
use crate::detector::utils::{run, file_exists, file_size, file_modified};
use std::collections::BTreeMap;
use std::path::Path;
use glob::glob;

pub fn collect(active_theme: &str) -> Initramfs {
    let mut tools = BTreeMap::new();
    for tool in ["update-initramfs", "dracut", "mkinitcpio"] {
        let path = run("which", &[tool]);
        let version = if !path.is_empty() { Some(run(tool, &["--version"])) } else { None };
        tools.insert(tool.to_string(), ToolInfo { 
            path: if path.is_empty() { None } else { Some(path) },
            version 
        });
    }

    let current_kernel = run("uname", &["-r"]);
    let mut images = Vec::new();
    if let Ok(paths) = glob("/boot/initrd.img-*") {
        for entry in paths.filter_map(|e| e.ok()) {
            let path = entry.to_string_lossy().into_owned();
            images.push(InitrdImage {
                path: path.clone(),
                size_bytes: file_size(&path),
                modified: file_modified(&path),
                is_current: path.contains(&current_kernel),
            });
        }
    }
    images.sort_by(|a, b| a.path.cmp(&b.path));

    let current_initrd = format!("/boot/initrd.img-{}", current_kernel);
    let mut analysis = InitrdAnalysis {
        path: current_initrd.clone(),
        exists: file_exists(&current_initrd),
        size_bytes: file_size(&current_initrd),
        modified: file_modified(&current_initrd),
        active_theme_modified: None,
        theme_newer_than_initrd: None,
        plymouth_files: Vec::new(),
        drm_kms_files: Vec::new(),
        bgrt_files: Vec::new(),
        active_theme_files: Vec::new(),
        active_theme_in_initramfs: false,
        error: None,
    };

    if analysis.exists {
        let theme_dir = format!("/usr/share/plymouth/themes/{}", active_theme);
        if file_exists(&theme_dir) {
            analysis.active_theme_modified = file_modified(&theme_dir);
            // Comparison is simplified but follows Python logic
            if let (Some(its), Some(tts)) = (fs_mtime(&current_initrd), fs_mtime(&theme_dir)) {
                analysis.theme_newer_than_initrd = Some(tts > its);
            }
        }

        let lsi = run("lsinitramfs", &[&current_initrd]);
        if !lsi.is_empty() {
            for line in lsi.lines() {
                let lower = line.to_lowercase();
                if lower.contains("plymouth") {
                    analysis.plymouth_files.push(line.to_string());
                    if !active_theme.is_empty() && line.contains(active_theme) {
                        analysis.active_theme_files.push(line.to_string());
                    }
                }
                if ["drm", "kms", "i915", "amdgpu", "nouveau"].iter().any(|&k| lower.contains(k)) {
                    analysis.drm_kms_files.push(line.to_string());
                }
                if lower.contains("bgrt") {
                    analysis.bgrt_files.push(line.to_string());
                }
            }
            analysis.active_theme_in_initramfs = !analysis.active_theme_files.is_empty();
        } else {
            analysis.error = Some("lsinitramfs failed or returned no output".to_string());
        }
    }

    Initramfs {
        tools,
        current_kernel,
        images,
        current_initrd_analysis: analysis,
    }
}

fn fs_mtime<P: AsRef<Path>>(path: P) -> Option<std::time::SystemTime> {
    std::fs::metadata(path).ok()?.modified().ok()
}
