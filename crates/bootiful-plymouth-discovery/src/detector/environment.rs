use crate::model::RuntimeEnvironment;
use crate::detector::utils::run;
use std::env;

pub fn collect() -> RuntimeEnvironment {
    let python_version = run("python3", &["--version"]).replace("Python ", "");
    let python_executable = run("which", &["python3"]);
    let uid = unsafe { libc::getuid() };
    let euid = unsafe { libc::geteuid() };
    let pid = std::process::id();

    RuntimeEnvironment {
        python_version,
        python_executable,
        uid,
        running_as_root: euid == 0,
        pid,
        path: env::var("PATH").ok(),
        display: env::var("DISPLAY").ok(),
        xdg_session_type: env::var("XDG_SESSION_TYPE").ok(),
        xdg_current_desktop: env::var("XDG_CURRENT_DESKTOP").ok(),
    }
}
