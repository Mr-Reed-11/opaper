use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

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
    fn list_outputs(&self) -> zbus::Result<Vec<String>>;
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
fn set_wallpaper(path: String) -> Result<(), String> {
    with_engine(|engine| engine.set_wallpaper(&path))
}

#[tauri::command]
fn pause() -> Result<(), String> {
    with_engine(|engine| engine.pause())
}

#[tauri::command]
fn resume() -> Result<(), String> {
    with_engine(|engine| engine.resume())
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
            set_wallpaper,
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
