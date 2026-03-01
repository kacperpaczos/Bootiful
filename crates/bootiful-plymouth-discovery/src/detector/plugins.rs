use crate::model::{Plugin, Renderer};
use crate::detector::utils::{run, file_size};
use std::collections::BTreeMap;
use glob::glob;

pub fn collect_plugins() -> BTreeMap<String, Plugin> {
    let mut result = BTreeMap::new();
    let plugin_dirs = glob("/usr/lib/*/plymouth/").unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.to_string_lossy().into_owned());
    
    let descriptions = get_plugin_descriptions();

    for d in plugin_dirs {
        let pattern = if d.ends_with('/') { format!("{}*.so", d) } else { format!("{}/*.so", d) };
        if let Ok(paths) = glob(&pattern) {
            for entry in paths.filter_map(|e| e.ok()) {
                let name = entry.file_stem().unwrap().to_string_lossy().into_owned();
                if !result.contains_key(&name) {
                    let so_path = entry.to_string_lossy().into_owned();
                    result.insert(name.clone(), Plugin {
                        so_path: so_path.clone(),
                        size_bytes: file_size(&so_path),
                        description: descriptions.get(&name).cloned().unwrap_or_else(|| format!("{} plugin", name)),
                        ldd: Some(run("ldd", &[&so_path]).lines().map(|s| s.to_string()).collect()),
                    });
                }
            }
        }
    }
    result
}

pub fn collect_renderers() -> BTreeMap<String, Renderer> {
    let mut result = BTreeMap::new();
    let renderer_dirs = glob("/usr/lib/*/plymouth/renderers/").unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.to_string_lossy().into_owned());
    
    let descriptions = get_renderer_descriptions();

    for d in renderer_dirs {
        let pattern = if d.ends_with('/') { format!("{}*.so", d) } else { format!("{}/*.so", d) };
        if let Ok(paths) = glob(&pattern) {
            for entry in paths.filter_map(|e| e.ok()) {
                let name = entry.file_stem().unwrap().to_string_lossy().into_owned();
                if !result.contains_key(&name) {
                    let so_path = entry.to_string_lossy().into_owned();
                    result.insert(name.clone(), Renderer {
                        so_path: so_path.clone(),
                        size_bytes: file_size(&so_path),
                        description: descriptions.get(&name).cloned().unwrap_or_else(|| format!("{} renderer", name)),
                    });
                }
            }
        }
    }
    result
}

fn get_plugin_descriptions() -> BTreeMap<String, String> {
    let mut m = BTreeMap::new();
    m.insert("details".into(), "Diagnostic – displays raw boot messages".into());
    m.insert("script".into(), "Scriptable – drives .script/.lua animations (most flexible)".into());
    m.insert("text".into(), "Text – simple TTY progress bar".into());
    m.insert("tribar".into(), "Tricolor progress bar".into());
    m.insert("two-step".into(), "Two-phase: static logo then animated throbber".into());
    m.insert("label".into(), "Label rendering (required by spinner/two-step)".into());
    m.insert("label-pango".into(), "Pango-based label (Ubuntu/Mint default)".into());
    m.insert("ubuntu-text".into(), "Ubuntu text mode plugin".into());
    m
}

fn get_renderer_descriptions() -> BTreeMap<String, String> {
    let mut m = BTreeMap::new();
    m.insert("drm".into(), "Preferred – DRM/KMS (/dev/dri/card0)".into());
    m.insert("frame-buffer".into(), "Fallback – /dev/fb0 directly".into());
    m.insert("x11".into(), "X11 test renderer (development only)".into());
    m
}
