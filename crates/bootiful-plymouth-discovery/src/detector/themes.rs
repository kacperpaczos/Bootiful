use crate::model::{ActiveTheme, AvailableTheme, PluginSo, AssetInfo, ScriptFileDetail};
use crate::detector::utils::{run, read_file, file_exists, file_size, sha256_short, sha256_full};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn collect_active_detail(name: &str) -> ActiveTheme {
    let mut result = ActiveTheme {
        name: name.to_string(),
        found: false,
        theme_dir: None,
        plymouth_file: None,
        plymouth_file_content: None,
        plugin: None,
        script_file: None,
        image_dir: None,
        extra_sections: BTreeMap::new(),
        plugin_so: PluginSo { path: None, size_bytes: None, ldd: None },
        assets: Vec::new(),
        script_file_detail: None,
    };

    if name.is_empty() {
        return result;
    }

    let search_roots = ["/usr/share/plymouth/themes", "/usr/local/share/plymouth/themes"];
    let mut pfile = None;
    let mut tdir = None;

    for root in search_roots {
        let path = Path::new(root).join(name);
        let pf = path.join(format!("{}.plymouth", name));
        if pf.exists() {
            tdir = Some(path);
            pfile = Some(pf);
            break;
        }
    }

    let pfile = match pfile {
        Some(p) => p,
        None => return result,
    };
    let tdir = tdir.unwrap();

    result.found = true;
    result.theme_dir = Some(tdir.to_string_lossy().into_owned());
    result.plymouth_file = Some(pfile.to_string_lossy().into_owned());
    
    let content = read_file(&pfile);
    result.plymouth_file_content = Some(content.clone());

    let ini = parse_ini(&content);
    if let Some(theme_section) = ini.get("Plymouth Theme") {
        result.plugin = theme_section.get("ModuleName").cloned();
        result.script_file = theme_section.get("ScriptFile").cloned();
        result.image_dir = theme_section.get("ImageDir").cloned();
    }
    
    result.extra_sections = ini.into_iter()
        .filter(|(k, _)| k != "Plymouth Theme")
        .collect();

    // Plugin .so
    if let Some(ref plugin_name) = result.plugin {
        let so_candidates = [
            format!("/usr/lib/x86_64-linux-gnu/plymouth/{}.so", plugin_name),
            format!("/usr/lib/plymouth/{}.so", plugin_name),
        ];
        for so in so_candidates {
            if file_exists(&so) {
                result.plugin_so.path = Some(so.clone());
                result.plugin_so.size_bytes = file_size(&so);
                result.plugin_so.ldd = Some(run("ldd", &[&so]).lines().map(|l| l.to_string()).collect());
                break;
            }
        }
    }

    // Assets
    let image_exts = ["png", "jpg", "svg", "bmp"];
    let mut assets = Vec::new();
    for entry in WalkDir::new(&tdir).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                if image_exts.contains(&ext.to_lowercase().as_str()) {
                    assets.push(AssetInfo {
                        file: entry.file_name().to_string_lossy().into_owned(),
                        size_bytes: file_size(entry.path()),
                        sha256_prefix: sha256_short(entry.path()),
                    });
                }
            }
        }
    }
    assets.sort_by(|a, b| a.file.cmp(&b.file));
    result.assets = assets;

    // Script file detail
    if let Some(ref sf) = result.script_file {
        let sp = tdir.join(sf);
        if sp.exists() {
            result.script_file_detail = Some(ScriptFileDetail {
                path: sp.to_string_lossy().into_owned(),
                exists: true,
                size_bytes: file_size(&sp),
                sha256: sha256_full(&sp),
            });
        }
    }

    result
}

pub fn scan_all(active_name: &str) -> BTreeMap<String, AvailableTheme> {
    let mut themes = BTreeMap::new();
    let search_roots = [
        "/usr/share/plymouth/themes",
        "/usr/local/share/plymouth/themes",
        "/opt/plymouth/themes",
    ];

    let current_kernel = run("uname", &["-r"]);
    let initrd = format!("/boot/initrd.img-{}", current_kernel);
    let initramfs_contents = if file_exists(&initrd) {
        run("lsinitramfs", &[&initrd])
    } else {
        String::new()
    };

    for root in search_roots {
        let root_path = Path::new(root);
        if !root_path.exists() { continue; }
        
        if let Ok(entries) = std::fs::read_dir(root_path) {
            for entry in entries.filter_map(|e| e.ok()) {
                if !entry.file_type().unwrap().is_dir() { continue; }
                
                let name = entry.file_name().to_string_lossy().into_owned();
                let dir = entry.path();
                let mut pfile = dir.join(format!("{}.plymouth", name));
                
                if !pfile.exists() {
                    if let Ok(mut sub_entries) = std::fs::read_dir(&dir) {
                        if let Some(found) = sub_entries.filter_map(|e| e.ok())
                            .find(|e| e.path().extension().map(|ext| ext == "plymouth").unwrap_or(false)) {
                            pfile = found.path();
                        }
                    }
                }

                let mut theme_entry = AvailableTheme {
                    dir: dir.to_string_lossy().into_owned(),
                    plymouth_file: if pfile.exists() { Some(pfile.to_string_lossy().into_owned()) } else { None },
                    is_active: name == active_name,
                    in_initramfs: if initramfs_contents.is_empty() { None } else { Some(initramfs_contents.contains(&name)) },
                    name: None,
                    description: None,
                    plugin: None,
                    script_file: None,
                    image_dir: None,
                    extra_sections: BTreeMap::new(),
                    missing_files: None,
                    error: None,
                };

                if pfile.exists() {
                    let content = read_file(&pfile);
                    let ini = parse_ini(&content);
                    if let Some(ts) = ini.get("Plymouth Theme") {
                        theme_entry.name = ts.get("Name").cloned();
                        theme_entry.description = ts.get("Description").cloned();
                        theme_entry.plugin = ts.get("ModuleName").cloned();
                        theme_entry.script_file = ts.get("ScriptFile").cloned();
                        theme_entry.image_dir = ts.get("ImageDir").cloned();
                    }
                    theme_entry.extra_sections = ini.into_iter()
                        .filter(|(k, _)| k != "Plymouth Theme")
                        .collect();
                    
                    // Check missing files
                    let mut missing = Vec::new();
                    if let Some(ref sf) = theme_entry.script_file {
                        let sp = if Path::new(sf).is_absolute() { PathBuf::from(sf) } else { dir.join(sf) };
                        if !sp.exists() { missing.push(format!("ScriptFile: {}", sp.display())); }
                    }
                    if let Some(ref imd) = theme_entry.image_dir {
                        let ip = if Path::new(imd).is_absolute() { PathBuf::from(imd) } else { dir.join(imd) };
                        if !ip.exists() { missing.push(format!("ImageDir: {}", ip.display())); }
                    }
                    if !missing.is_empty() { theme_entry.missing_files = Some(missing); }
                } else {
                    theme_entry.error = Some("no .plymouth file found".to_string());
                }

                themes.insert(name, theme_entry);
            }
        }
    }
    themes
}

fn parse_ini(content: &str) -> BTreeMap<String, BTreeMap<String, String>> {
    let mut result = BTreeMap::new();
    let mut current_section = String::new();
    
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        
        if line.starts_with('[') && line.ends_with(']') {
            current_section = line[1..line.len()-1].to_string();
            result.insert(current_section.clone(), BTreeMap::new());
        } else if line.contains('=') && !current_section.is_empty() {
            let mut parts = line.splitn(2, '=');
            if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
                if let Some(sec) = result.get_mut(&current_section) {
                    sec.insert(k.trim().to_string(), v.trim().to_string());
                }
            }
        }
    }
    result
}
