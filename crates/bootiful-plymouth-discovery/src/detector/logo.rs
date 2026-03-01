use crate::model::{DistributionLogo, LogoInfo};
use crate::detector::utils::{file_exists, resolve_path, file_size};
use std::collections::BTreeMap;
use std::path::Path;

pub fn collect() -> DistributionLogo {
    let candidates = [
        ("/usr/share/plymouth/ubuntu-logo.png", "ubuntu_logo_png", "Ubuntu OEM logo (used by 'special://logo')"),
        ("/usr/share/pixmaps/mint-logo.png", "mint_logo_png", "Linux Mint logo"),
        ("/usr/share/pixmaps/distributor-logo.png", "distributor_logo_png", "Generic distributor logo (fallback)"),
        ("/etc/alternatives/distributor-logo", "distributor_logo", "update-alternatives distributor-logo link"),
    ];

    let mut logos = BTreeMap::new();
    for (path, key, desc) in candidates {
        let exists = file_exists(path);
        let is_symlink = Path::new(path).is_symlink();
        
        logos.insert(key.to_string(), LogoInfo {
            path: path.to_string(),
            exists,
            is_symlink: if exists || is_symlink { Some(is_symlink) } else { None },
            resolved: if exists || is_symlink { Some(resolve_path(path)) } else { None },
            description: desc.to_string(),
            size_bytes: if exists { file_size(path) } else { None },
        });
    }

    DistributionLogo {
        note: "In themes, 'special://logo' resolves to the distribution OEM logo at runtime.".into(),
        logos,
    }
}
