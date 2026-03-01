use std::process::Command;
use std::path::Path;
use std::fs;
use sha2::{Sha256, Digest};
use std::io::Read;

pub fn run(cmd: &str, args: &[&str]) -> String {
    Command::new(cmd)
        .args(args)
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

pub fn run_with_status(cmd: &str, args: &[&str]) -> (i32, String, String) {
    match Command::new(cmd).args(args).output() {
        Ok(o) => (
            o.status.code().unwrap_or(-1),
            String::from_utf8_lossy(&o.stdout).trim().to_string(),
            String::from_utf8_lossy(&o.stderr).trim().to_string(),
        ),
        Err(e) => (-1, "".into(), e.to_string()),
    }
}

pub fn read_file<P: AsRef<Path>>(path: P) -> String {
    fs::read_to_string(path).unwrap_or_default().trim().to_string()
}

pub fn file_exists<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().exists()
}

pub fn file_size<P: AsRef<Path>>(path: P) -> Option<u64> {
    fs::metadata(path).ok().map(|m| m.len())
}

pub fn file_modified<P: AsRef<Path>>(path: P) -> Option<String> {
    use chrono::{DateTime, Local};
    fs::metadata(path).ok()?.modified().ok().map(|m| {
        let dt: DateTime<Local> = m.into();
        dt.format("%Y-%m-%dT%H:%M:%S").to_string()
    })
}

pub fn sha256_short<P: AsRef<Path>>(path: P) -> Option<String> {
    let mut file = fs::File::open(path).ok()?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 65536];
    while let Ok(n) = file.read(&mut buffer) {
        if n == 0 { break; }
        hasher.update(&buffer[..n]);
    }
    let result = hasher.finalize();
    Some(format!("{:x}", result)[..16].to_string())
}

pub fn sha256_full<P: AsRef<Path>>(path: P) -> Option<String> {
    let mut file = fs::File::open(path).ok()?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 65536];
    while let Ok(n) = file.read(&mut buffer) {
        if n == 0 { break; }
        hasher.update(&buffer[..n]);
    }
    let result = hasher.finalize();
    Some(format!("{:x}", result))
}

pub fn resolve_path<P: AsRef<Path>>(path: P) -> String {
    fs::read_link(&path)
        .ok()
        .map(|p| {
            if p.is_absolute() {
                p.to_string_lossy().into_owned()
            } else {
                path.as_ref().parent()
                    .unwrap_or(Path::new("."))
                    .join(p)
                    .to_string_lossy()
                    .into_owned()
            }
        })
        .unwrap_or_else(|| path.as_ref().to_string_lossy().into_owned())
}

pub fn symlink_chain<P: AsRef<Path>>(start: P) -> Vec<crate::model::SymlinkStep> {
    use crate::model::SymlinkStep;
    let mut result = Vec::new();
    let mut current = start.as_ref().to_path_buf();
    let mut seen = std::collections::HashSet::new();

    loop {
        let is_symlink = current.is_symlink();
        let exists = current.exists();
        let mut step = SymlinkStep {
            path: current.to_string_lossy().into_owned(),
            is_symlink,
            exists,
            target: None,
        };

        if is_symlink {
            if let Ok(target) = fs::read_link(&current) {
                let abs_target = if target.is_absolute() {
                    target
                } else {
                    current.parent().unwrap_or(Path::new(".")).join(target)
                };
                step.target = Some(abs_target.to_string_lossy().into_owned());
                result.push(step);
                
                let target_key = abs_target.to_string_lossy().into_owned();
                if seen.contains(&target_key) {
                    break;
                }
                seen.insert(target_key);
                current = abs_target;
            } else {
                result.push(step);
                break;
            }
        } else {
            result.push(step);
            break;
        }
    }
    result
}
