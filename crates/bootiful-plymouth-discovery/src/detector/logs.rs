use crate::model::{Logs, RunPlymouthLogInfo, LogFileInfo, JournalctlInfo};
use crate::detector::utils::{run, file_exists, read_file};
use std::collections::BTreeMap;

pub fn collect() -> Logs {
    let run_plymouth = "/run/plymouth";
    let run_ply_info = RunPlymouthLogInfo {
        path: run_plymouth.into(),
        exists: file_exists(run_plymouth),
        note: "See runtime_state.run_plymouth_files for details".into(),
    };

    Logs {
        run_plymouth: run_ply_info,
        boot_log: collect_log_file("/var/log/boot.log", "plymouth_lines", 0),
        syslog: collect_log_file("/var/log/syslog", "plymouth_lines_last_20", 20),
        kern_log: collect_log_file("/var/log/kern.log", "plymouth_lines_last_10", 10),
        journalctl_current_boot: JournalctlInfo {
            plymouth_lines_last_40: run("journalctl", &["-b", "--no-pager", "-q"])
                .lines()
                .filter(|l| l.to_lowercase().contains("plymouth"))
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .take(40)
                .collect::<Vec<_>>()
                .into_iter()
                .rev()
                .collect(),
        },
    }
}

fn collect_log_file(path: &str, field_name: &str, last_n: usize) -> LogFileInfo {
    let exists = file_exists(path);
    let mut lines = Vec::new();
    if exists {
        let content = read_file(path);
        let all_plymouth: Vec<String> = content.lines()
            .filter(|l| l.to_lowercase().contains("plymouth"))
            .map(|s| s.to_string())
            .collect();
        
        if last_n > 0 {
            lines = all_plymouth.into_iter().rev().take(last_n).collect::<Vec<_>>().into_iter().rev().collect();
        } else {
            lines = all_plymouth;
        }
    }

    let mut map = BTreeMap::new();
    map.insert(field_name.to_string(), lines);

    LogFileInfo {
        path: path.to_string(),
        exists,
        lines: map,
    }
}
