use crate::model::{Graphics, FramebufferDevice, DrmDevice, BgrtInfo};
use crate::detector::utils::{run, read_file, file_exists};
use glob::glob;
use std::path::Path;

pub fn collect() -> Graphics {
    let mut framebuffer_devices = Vec::new();
    if let Ok(paths) = glob("/dev/fb*") {
        for entry in paths.filter_map(|e| e.ok()) {
            let name = entry.file_name().unwrap_or_default().to_string_lossy();
            let sys_path = format!("/sys/class/graphics/{}", name);
            framebuffer_devices.push(FramebufferDevice {
                device: entry.to_string_lossy().into_owned(),
                virtual_size: if file_exists(&sys_path) { Some(read_file(format!("{}/virtual_size", sys_path))) } else { None },
                bits_per_pixel: if file_exists(&sys_path) { Some(read_file(format!("{}/bits_per_pixel", sys_path))) } else { None },
                stride: if file_exists(&sys_path) { Some(read_file(format!("{}/stride", sys_path))) } else { None },
            });
        }
    }

    let graphics_sys_entries = glob("/sys/class/graphics/*").unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.to_string_lossy().into_owned())
        .collect();

    let mut drm_devices = Vec::new();
    if let Ok(paths) = glob("/sys/class/drm/card*") {
        for entry in paths.filter_map(|e| e.ok()) {
            let name = entry.file_name().unwrap_or_default().to_string_lossy().into_owned();
            let enabled = if file_exists(entry.join("enabled")) { Some(read_file(entry.join("enabled"))) } else { None };
            let driver = entry.join("device/driver").read_link().ok()
                .and_then(|p| {
                    p.file_name().map(|f| f.to_string_lossy().into_owned())
                });
            
            drm_devices.push(DrmDevice { name, enabled, driver });
        }
    }

    let lsmod = run("lsmod", &[]);
    let graphics_mods = ["drm", "i915", "amdgpu", "radeon", "nouveau", "nvidia", "vmwgfx", "vboxvideo", "virtio_gpu", "ast", "simpledrm", "efifb", "vesafb", "bochs", "vkms", "mgag200"];
    let mut loaded_graphics_modules = Vec::new();
    for m in lsmod.lines() {
        if let Some(name) = m.split_whitespace().next() {
            if graphics_mods.contains(&name) {
                loaded_graphics_modules.push(name.to_string());
            }
        }
    }

    let bgrt_path = "/sys/firmware/acpi/bgrt";
    let bgrt = BgrtInfo {
        present: file_exists(bgrt_path),
        xoffset: if file_exists(format!("{}/xoffset", bgrt_path)) { Some(read_file(format!("{}/xoffset", bgrt_path))) } else { None },
        yoffset: if file_exists(format!("{}/yoffset", bgrt_path)) { Some(read_file(format!("{}/yoffset", bgrt_path))) } else { None },
        status: if file_exists(format!("{}/status", bgrt_path)) { Some(read_file(format!("{}/status", bgrt_path))) } else { None },
        r#type: if file_exists(format!("{}/type", bgrt_path)) { Some(read_file(format!("{}/type", bgrt_path))) } else { None },
    };

    Graphics {
        framebuffer_devices,
        graphics_sys_entries,
        drm_devices,
        loaded_graphics_modules,
        simpledrm_loaded: lsmod.contains("simpledrm"),
        efifb_loaded: lsmod.contains("efifb"),
        uefi: file_exists("/sys/firmware/efi"),
        bgrt,
    }
}
