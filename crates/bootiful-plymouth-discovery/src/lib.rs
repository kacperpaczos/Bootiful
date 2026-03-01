pub mod model;
pub mod detector;
pub mod error;

pub use model::PlymouthConfig;
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

pub fn collect_all() -> Result<PlymouthConfig> {
    let environment = detector::environment::collect();
    let system = detector::system::collect();
    let packages = detector::packages::collect();
    let global_config = detector::config::collect();
    let active_theme = detector::themes::collect_active_detail(&global_config.active_theme);
    let available_themes = detector::themes::scan_all(&global_config.active_theme);
    let available_plugins = detector::plugins::collect_plugins();
    let available_renderers = detector::plugins::collect_renderers();
    let distribution_logo = detector::logo::collect();
    let initramfs = detector::initramfs::collect(&global_config.active_theme);
    let bootloader = detector::bootloader::collect();
    let runtime_state = detector::runtime::collect();
    let graphics = detector::graphics::collect();
    let logs = detector::logs::collect();

    let meta = model::Meta {
        generated_at: chrono::Local::now().format("%Y-%m-%dT%H:%M:%S").to_string(),
        generator: "bootiful-plymouth-discovery".into(),
        python_version: environment.python_version.clone(),
        uid: environment.uid,
        sections: vec![
            "runtime_environment".into(), "system_context".into(), "packages".into(),
            "global_config".into(), "active_theme".into(), "available_themes".into(),
            "available_plugins".into(), "available_renderers".into(),
            "distribution_logo".into(), "initramfs".into(), "bootloader".into(),
            "runtime_state".into(), "graphics".into(), "logs".into(), "consistency_check".into(),
        ],
    };

    let mut config = PlymouthConfig {
        _meta: meta,
        runtime_environment: environment,
        system_context: system,
        packages,
        global_config,
        active_theme,
        available_themes,
        available_plugins,
        available_renderers,
        distribution_logo,
        initramfs,
        bootloader,
        runtime_state,
        graphics,
        logs,
        consistency_check: model::ConsistencyCheck {
            all_ok: false,
            passed: vec![],
            issues: vec![],
        },
    };

    config.consistency_check = detector::consistency::check(&config);

    Ok(config)
}
