use crate::constants::*;
use tauri::{AppHandle, Manager};

pub fn setup_tray(app: &mut tauri::App) {
    use tauri::tray::{TrayIcon, TrayIconBuilder};
    use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};

    let handle = app.handle();
    
    let show_i = MenuItem::with_id(handle, "show", "显示", true, None::<&str>).unwrap();
    let sep_i = PredefinedMenuItem::separator(handle).unwrap();
    let quit_i = MenuItem::with_id(handle, "quit", "退出", true, None::<&str>).unwrap();
    
    let menu = Menu::with_items(handle, &[&show_i, &sep_i, &quit_i]).unwrap();

    let _tray = TrayIconBuilder::with_id("main-tray")
        .tooltip(APP_NAME)
        .icon(handle.default_window_icon().unwrap().clone())
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray: &TrayIcon<tauri::Wry>, event| {
            if let tauri::tray::TrayIconEvent::Click {
                button: tauri::tray::MouseButton::Left,
                button_state: tauri::tray::MouseButtonState::Up,
                ..
            } = event
            {
                let app = tray.app_handle();
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        })
        .on_menu_event(|app: &AppHandle, event| {
            match event.id.as_ref() {
                "show" => {
                    if let Some(window) = app.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
                "quit" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
        .build(app);
}
