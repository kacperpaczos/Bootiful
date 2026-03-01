use crate::model::{Packages, PackageEntry};
use crate::detector::utils::{run, run_with_status};
use std::collections::BTreeMap;

pub fn collect() -> Packages {
    let mut packages_map = BTreeMap::new();
    let key_pkgs = ["plymouth", "plymouth-themes", "plymouth-label", "python3-plymouth"];

    for pkg in key_pkgs {
        let (rc_ver, ver, _) = run_with_status("dpkg-query", &["-W", "-f=${Version}", pkg]);
        let (_, status, _) = run_with_status("dpkg-query", &["-W", "-f=${Status}", pkg]);
        
        packages_map.insert(pkg.to_string(), PackageEntry {
            version: if rc_ver == 0 && !ver.is_empty() { Some(ver) } else { None },
            status: if status.is_empty() { "not-installed".to_string() } else { status },
            description: None,
        });
    }

    // Auto-discover other *plymouth* packages
    let dpkg_l = run("dpkg", &["-l", "*plymouth*"]);
    for line in dpkg_l.lines() {
        if line.starts_with("ii") || line.starts_with("iF") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 3 {
                let name = parts[1].split(':').next().unwrap_or("").to_string();
                if !packages_map.contains_key(&name) {
                    packages_map.insert(name, PackageEntry {
                        version: Some(parts[2].to_string()),
                        status: "install ok installed".to_string(),
                        description: if parts.len() > 4 { Some(parts[4..].join(" ")) } else { None },
                    });
                }
            }
        }
    }

    let _plymouth_file_list = run("dpkg", &["-L", "plymouth"])
        .lines()
        .map(|s| s.to_string())
        .collect();

    let _apt_cache_policy = run("apt-cache", &["policy", "plymouth"]);

    Packages {
        packages: packages_map,
        _plymouth_file_list,
        _apt_cache_policy,
        _apt_cache_show: parse_apt_cache_show("plymouth"),
    }
}

fn parse_apt_cache_show(pkg: &str) -> BTreeMap<String, String> {
    let mut result = BTreeMap::new();
    let content = run("apt-cache", &["show", pkg]);
    let fields = ["Package", "Version", "Section", "Origin", "Maintainer", "Original-Maintainer", "Installed-Size"];
    
    for line in content.lines() {
        for field in fields {
            if line.starts_with(field) && line.contains(':') {
                let mut parts = line.splitn(2, ':');
                if let (Some(_), Some(v)) = (parts.next(), parts.next()) {
                    result.insert(field.to_string(), v.trim().to_string());
                }
            }
        }
    }
    result
}
