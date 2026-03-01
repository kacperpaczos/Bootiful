use serde::{Serialize, Deserialize};
use std::collections::BTreeMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlymouthConfig {
    pub _meta: Meta,
    pub runtime_environment: RuntimeEnvironment,
    pub system_context: SystemContext,
    pub packages: Packages,
    pub global_config: GlobalConfig,
    pub active_theme: ActiveTheme,
    pub available_themes: BTreeMap<String, AvailableTheme>,
    pub available_plugins: BTreeMap<String, Plugin>,
    pub available_renderers: BTreeMap<String, Renderer>,
    pub distribution_logo: DistributionLogo,
    pub initramfs: Initramfs,
    pub bootloader: Bootloader,
    pub runtime_state: RuntimeState,
    pub graphics: Graphics,
    pub logs: Logs,
    pub consistency_check: ConsistencyCheck,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Meta {
    pub generated_at: String,
    pub generator: String,
    pub python_version: String,
    pub uid: u32,
    pub sections: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RuntimeEnvironment {
    pub python_version: String,
    pub python_executable: String,
    pub uid: u32,
    pub running_as_root: bool,
    pub pid: u32,
    pub path: Option<String>,
    pub display: Option<String>,
    pub xdg_session_type: Option<String>,
    pub xdg_current_desktop: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemContext {
    pub uname_full: String,
    pub uname_machine: String,
    pub dpkg_architecture: String,
    pub os_release: BTreeMap<String, String>,
    pub lsb_release: BTreeMap<String, String>,
    pub hostname: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Packages {
    #[serde(flatten)]
    pub packages: BTreeMap<String, PackageEntry>,
    pub _plymouth_file_list: Vec<String>,
    pub _apt_cache_policy: String,
    pub _apt_cache_show: BTreeMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PackageEntry {
    pub version: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GlobalConfig {
    pub etc_plymouth_directory: DirectoryInfo,
    pub daemon_defaults_file: ConfigFile,
    pub daemon_conf_file: ConfigFile,
    pub effective_daemon_config: BTreeMap<String, serde_json::Value>,
    pub active_theme: String,
    pub update_alternatives_display: String,
    pub update_alternatives_query: String,
    pub symlink_chains: BTreeMap<String, Vec<SymlinkStep>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DirectoryInfo {
    pub exists: bool,
    pub files: Vec<FileInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FileInfo {
    pub path: String,
    pub size_bytes: Option<u64>,
    pub modified: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConfigFile {
    pub path: String,
    pub present: bool,
    pub content: Option<String>,
    pub values: BTreeMap<String, Option<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SymlinkStep {
    pub path: String,
    pub is_symlink: bool,
    pub exists: bool,
    pub target: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActiveTheme {
    pub name: String,
    pub found: bool,
    pub theme_dir: Option<String>,
    pub plymouth_file: Option<String>,
    pub plymouth_file_content: Option<String>,
    pub plugin: Option<String>,
    pub script_file: Option<String>,
    pub image_dir: Option<String>,
    pub extra_sections: BTreeMap<String, BTreeMap<String, String>>,
    pub plugin_so: PluginSo,
    pub assets: Vec<AssetInfo>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_file_detail: Option<ScriptFileDetail>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PluginSo {
    pub path: Option<String>,
    pub size_bytes: Option<u64>,
    pub ldd: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AssetInfo {
    pub file: String,
    pub size_bytes: Option<u64>,
    pub sha256_prefix: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScriptFileDetail {
    pub path: String,
    pub exists: bool,
    pub size_bytes: Option<u64>,
    pub sha256: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AvailableTheme {
    pub dir: String,
    pub plymouth_file: Option<String>,
    pub is_active: bool,
    pub in_initramfs: Option<bool>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub plugin: Option<String>,
    pub script_file: Option<String>,
    pub image_dir: Option<String>,
    pub extra_sections: BTreeMap<String, BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missing_files: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Plugin {
    pub so_path: String,
    pub size_bytes: Option<u64>,
    pub description: String,
    pub ldd: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Renderer {
    pub so_path: String,
    pub size_bytes: Option<u64>,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DistributionLogo {
    pub note: String,
    #[serde(flatten)]
    pub logos: BTreeMap<String, LogoInfo>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogoInfo {
    pub path: String,
    pub exists: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_symlink: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved: Option<String>,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size_bytes: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Initramfs {
    pub tools: BTreeMap<String, ToolInfo>,
    pub current_kernel: String,
    pub images: Vec<InitrdImage>,
    pub current_initrd_analysis: InitrdAnalysis,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ToolInfo {
    pub path: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InitrdImage {
    pub path: String,
    pub size_bytes: Option<u64>,
    pub modified: Option<String>,
    pub is_current: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InitrdAnalysis {
    pub path: String,
    pub exists: bool,
    pub size_bytes: Option<u64>,
    pub modified: Option<String>,
    pub active_theme_modified: Option<String>,
    pub theme_newer_than_initrd: Option<bool>,
    pub plymouth_files: Vec<String>,
    pub drm_kms_files: Vec<String>,
    pub bgrt_files: Vec<String>,
    pub active_theme_files: Vec<String>,
    pub active_theme_in_initramfs: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bootloader {
    #[serde(rename = "type")]
    pub bootloader_type: String,
    pub grub: GrubConfig,
    pub systemd_boot: SystemdBootConfig,
    pub active_cmdline: ActiveCmdline,
    pub kernel_entries: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GrubConfig {
    pub present: bool,
    pub default_config_path: Option<String>,
    pub cfg_path: Option<String>,
    pub cmdline_linux_default: String,
    pub cmdline_linux: String,
    pub non_comment_lines: Vec<String>,
    pub splash_plymouth_entries_in_cfg: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SystemdBootConfig {
    pub present: bool,
    pub status: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ActiveCmdline {
    pub raw: String,
    pub parameters: BTreeMap<String, bool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RuntimeState {
    pub version: Option<String>,
    pub daemon_running: bool,
    pub daemon_mode: String,
    pub valid_modes: Vec<String>,
    pub mode_control_commands: BTreeMap<String, String>,
    pub units_list: Vec<String>,
    pub unit_states: BTreeMap<String, UnitState>,
    pub plymouth_quit_wait_properties: BTreeMap<String, String>,
    pub plymouth_start_unit_file: Option<String>,
    pub display_manager: DisplayManagerInfo,
    pub run_plymouth_files: Vec<RunPlymouthFile>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UnitState {
    pub active: String,
    pub enabled: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DisplayManagerInfo {
    pub service: Option<String>,
    pub active: bool,
    pub depends_on_plymouth_quit_wait: bool,
    pub after_property: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RunPlymouthFile {
    pub path: String,
    pub size_bytes: Option<u64>,
    pub content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Graphics {
    pub framebuffer_devices: Vec<FramebufferDevice>,
    pub graphics_sys_entries: Vec<String>,
    pub drm_devices: Vec<DrmDevice>,
    pub loaded_graphics_modules: Vec<String>,
    pub simpledrm_loaded: bool,
    pub efifb_loaded: bool,
    pub uefi: bool,
    pub bgrt: BgrtInfo,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FramebufferDevice {
    pub device: String,
    pub virtual_size: Option<String>,
    pub bits_per_pixel: Option<String>,
    pub stride: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DrmDevice {
    pub name: String,
    pub enabled: Option<String>,
    pub driver: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BgrtInfo {
    pub present: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub xoffset: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub yoffset: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Logs {
    pub run_plymouth: RunPlymouthLogInfo,
    pub boot_log: LogFileInfo,
    pub syslog: LogFileInfo,
    pub kern_log: LogFileInfo,
    pub journalctl_current_boot: JournalctlInfo,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RunPlymouthLogInfo {
    pub path: String,
    pub exists: bool,
    pub note: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LogFileInfo {
    pub path: String,
    pub exists: bool,
    #[serde(flatten)]
    pub lines: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct JournalctlInfo {
    pub plymouth_lines_last_40: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ConsistencyCheck {
    pub all_ok: bool,
    pub passed: Vec<String>,
    pub issues: Vec<String>,
}
