use crate::model::SystemContext;
use crate::detector::utils::{run, read_file};
use std::collections::BTreeMap;

pub fn collect() -> SystemContext {
    let uname_full = run("uname", &["-a"]);
    let uname_machine = run("uname", &["-m"]);
    let dpkg_architecture = run("dpkg", &["--print-architecture"]);
    let hostname = run("hostname", &[]);

    SystemContext {
        uname_full,
        uname_machine,
        dpkg_architecture,
        os_release: parse_os_release(),
        lsb_release: parse_lsb_release(),
        hostname,
    }
}

fn parse_os_release() -> BTreeMap<String, String> {
    let mut result = BTreeMap::new();
    let content = read_file("/etc/os-release");
    for line in content.lines() {
        if line.starts_with('#') || !line.contains('=') {
            continue;
        }
        let mut parts = line.splitn(2, '=');
        if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
            let key = k.trim().to_string();
            let value = v.trim().trim_matches('"').trim_matches('\'').to_string();
            result.insert(key, value);
        }
    }
    result
}

fn parse_lsb_release() -> BTreeMap<String, String> {
    let mut result = BTreeMap::new();
    let content = run("lsb_release", &["-a"]);
    for line in content.lines() {
        if !line.contains(':') {
            continue;
        }
        let mut parts = line.splitn(2, ':');
        if let (Some(k), Some(v)) = (parts.next(), parts.next()) {
            result.insert(k.trim().to_string(), v.trim().to_string());
        }
    }
    result
}
