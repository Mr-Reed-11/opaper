use std::path::Path;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use serde::Serialize;
use tauri::{
    AppHandle, Manager, WindowEvent,
    image::Image,
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::TrayIconBuilder,
};
use zbus::{blocking::Connection, proxy};

#[proxy(
    interface = "org.opaper.Engine",
    default_service = "org.opaper.Engine",
    default_path = "/org/opaper/Engine"
)]
trait Engine {
    fn set_wallpaper(&self, path: &str) -> zbus::Result<()>;
    fn set_wallpaper_on_output(&self, path: &str, output: &str) -> zbus::Result<()>;
    fn set_param(&self, slot: u8, r: f64, g: f64, b: f64, a: f64, output: &str) -> zbus::Result<()>;
    fn set_fps_limit(&self, fps: u32) -> zbus::Result<()>;
    fn get_active_wallpaper(&self) -> zbus::Result<String>;
    fn get_manifest(&self) -> zbus::Result<String>;
    fn get_params(&self) -> zbus::Result<String>;
    fn list_outputs(&self) -> zbus::Result<Vec<String>>;
    fn set_pause_on_fullscreen(&self, enabled: bool) -> zbus::Result<()>;
    fn pause(&self) -> zbus::Result<()>;
    fn resume(&self) -> zbus::Result<()>;
    fn quit(&self) -> zbus::Result<()>;
}

fn with_engine<T>(call: impl FnOnce(&EngineProxyBlocking) -> zbus::Result<T>) -> Result<T, String> {
    let connection = Connection::session().map_err(|e| format!("D-Bus: {e}"))?;
    let proxy = EngineProxyBlocking::new(&connection).map_err(|e| format!("engine: {e}"))?;
    call(&proxy).map_err(|e| format!("engine: {e}"))
}

#[tauri::command]
fn list_outputs() -> Result<Vec<String>, String> {
    with_engine(|engine| engine.list_outputs())
}

#[tauri::command]
fn set_wallpaper(path: String, output: Option<String>) -> Result<(), String> {
    with_engine(|engine| match &output {
        Some(output) => engine.set_wallpaper_on_output(&path, output),
        None => engine.set_wallpaper(&path),
    })
}

#[tauri::command]
fn set_param(slot: u8, r: f64, g: f64, b: f64, a: f64, output: Option<String>) -> Result<(), String> {
    with_engine(|engine| engine.set_param(slot, r, g, b, a, output.as_deref().unwrap_or("")))
}

#[tauri::command]
fn set_fps_limit(fps: u32) -> Result<(), String> {
    with_engine(|engine| engine.set_fps_limit(fps))
}

#[tauri::command]
fn get_active_wallpaper() -> Result<String, String> {
    with_engine(|engine| engine.get_active_wallpaper())
}

#[tauri::command]
fn get_manifest() -> Result<String, String> {
    with_engine(|engine| engine.get_manifest())
}

#[tauri::command]
fn get_params() -> Result<String, String> {
    with_engine(|engine| engine.get_params())
}

#[tauri::command]
fn set_pause_on_fullscreen(enabled: bool) -> Result<(), String> {
    with_engine(|engine| engine.set_pause_on_fullscreen(enabled))
}

#[tauri::command]
fn pause() -> Result<(), String> {
    with_engine(|engine| engine.pause())
}

#[tauri::command]
fn resume() -> Result<(), String> {
    with_engine(|engine| engine.resume())
}

#[derive(Serialize, Clone)]
struct WallpaperInfo {
    name: String,
    description: String,
    author: String,
    path: String,
    colors: Vec<String>,
}

fn scan_wallpaper_dir(dir: &Path) -> Vec<WallpaperInfo> {
    let mut wallpapers = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else { return wallpapers };
    for entry in entries.flatten() {
        let path = entry.path();
        let manifest_path = path.join("manifest.json");
        if !manifest_path.is_file() { continue; }
        let Ok(raw) = std::fs::read_to_string(&manifest_path) else { continue };
        let Ok(value) = serde_json::from_str::<serde_json::Value>(&raw) else { continue };

        let name = value["name"].as_str().unwrap_or("").to_string();
        let description = value["description"].as_str().unwrap_or("").to_string();
        let author = value["author"].as_str().unwrap_or("").to_string();

        let mut colors = Vec::new();
        if let Some(params) = value["params"].as_object() {
            for (_, param) in params {
                if param["type"].as_str() == Some("color") {
                    if let Some(c) = param["default"].as_str() {
                        colors.push(c.to_string());
                    }
                }
            }
        }

        wallpapers.push(WallpaperInfo {
            name,
            description,
            author,
            path: path.to_string_lossy().to_string(),
            colors,
        });
    }
    wallpapers.sort_by(|a, b| a.name.cmp(&b.name));
    wallpapers
}

#[tauri::command]
fn list_wallpapers() -> Vec<WallpaperInfo> {
    let mut all = Vec::new();

    // User wallpapers (take priority over system)
    let home = std::env::var("HOME").unwrap_or_default();
    let xdg = std::env::var("XDG_DATA_HOME")
        .unwrap_or_else(|_| format!("{home}/.local/share"));
    all.extend(scan_wallpaper_dir(&Path::new(&xdg).join("opaper/wallpapers")));

    // System-wide examples (installed)
    all.extend(scan_wallpaper_dir(Path::new("/usr/share/opaper/examples")));

    // Deduplicate by name (user wins over system)
    all.sort_by(|a, b| a.name.cmp(&b.name));
    all.dedup_by(|a, b| a.name == b.name);
    all
}

#[tauri::command]
fn read_manifest(path: String) -> Result<String, String> {
    let manifest_path = Path::new(&path).join("manifest.json");
    std::fs::read_to_string(&manifest_path)
        .map_err(|e| format!("cannot read manifest: {e}"))
}

fn stop_engine() {
    let _ = with_engine(|engine| engine.quit());
}

#[tauri::command]
fn quit(app: AppHandle) {
    stop_engine();
    app.exit(0);
}

pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            list_outputs,
            list_wallpapers,
            read_manifest,
            set_wallpaper,
            set_param,
            set_fps_limit,
            set_pause_on_fullscreen,
            get_active_wallpaper,
            get_manifest,
            get_params,
            pause,
            resume,
            quit,
        ])
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                let _ = window.hide();
            }
        })
        .setup(|app| {
            let open = MenuItem::with_id(app, "open", "Abrir biblioteca", true, None::<&str>)?;
            let pause = MenuItem::with_id(app, "pause", "Pausar", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "Sair", true, None::<&str>)?;
            let separator = PredefinedMenuItem::separator(app)?;
            let menu = Menu::with_items(app, &[&open, &pause, &separator, &quit])?;

            let paused = Arc::new(AtomicBool::new(false));
            let toggle = pause.clone();
            let paused_flag = paused.clone();

            let icon = Image::from_bytes(include_bytes!("../icons/tray.png"))?;
            let tray = TrayIconBuilder::with_id("opaper-tray")
                .menu(&menu)
                .icon(icon)
                .on_menu_event(move |app, event| match event.id().as_ref() {
                    "open" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                    "pause" => {
                        let next = !paused_flag.load(Ordering::Relaxed);
                        let result = if next {
                            with_engine(|engine| engine.pause())
                        } else {
                            with_engine(|engine| engine.resume())
                        };
                        if result.is_ok() {
                            paused_flag.store(next, Ordering::Relaxed);
                            let _ = toggle.set_text(if next { "Retomar" } else { "Pausar" });
                        }
                    }
                    "quit" => {
                        stop_engine();
                        app.exit(0);
                    }
                    _ => {}
                })
                .build(app)?;

            app.manage(tray);
            app.manage(menu);
            app.manage(paused);
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("failed to run opaper");
}
