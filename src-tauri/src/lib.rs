use ddc_hi::{Ddc, Display};
use serde::Serialize;
use tauri::menu::Menu;
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, WindowEvent};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use tauri_plugin_notification::NotificationExt;
use tauri_plugin_positioner::{Position, WindowExt};

const FEATURE_CODE: u8 = 0x10;

#[derive(Serialize)]
struct DisplayInfo {
    id: String,
    name: String,
    brightness: u16,
}

#[tauri::command(async)]
fn list_displays() -> Result<Vec<DisplayInfo>, String> {
    let mut displays: Vec<DisplayInfo> = Vec::new();
    for mut display in Display::enumerate() {
        match display.update_capabilities() {
            Ok(_) => {}
            Err(e) => {
                eprintln!("failed to update capabilities: {}", e)
            }
        }

        let info = &display.info;
        let (Some(model), Some(manufacturer)) = (&info.model_name, &info.manufacturer_id) else {
            continue;
        };

        if model.is_empty() || manufacturer.is_empty() || model == "Generic PnP Monitor" {
            continue;
        }

        let brightness = match display.handle.get_vcp_feature(FEATURE_CODE) {
            Ok(val) => val.value(),
            Err(_) => continue,
        };

        displays.push(DisplayInfo {
            id: info.id.clone(),
            name: model.clone(),
            brightness,
        })
    }
    Ok(displays)
}

#[tauri::command]
fn set_brightness(id: String, input_value: u16) -> Result<(), String> {
    for mut display in Display::enumerate() {
        if display.info.id == id {
            return match display.handle.set_vcp_feature(FEATURE_CODE, input_value) {
                Ok(_) => {
                    println!("Brightness set to {} for {}", input_value, id);
                    Ok(())
                }
                Err(e) => Err(format!("Failed to set brightness: {}", e)),
            };
        }
    }

    Err("Display not found".to_string())
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|_app, _args, _cwd| {}))
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--flag1", "--flag2"]),
        ))
        .plugin(tauri_plugin_positioner::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![list_displays, set_brightness])
        .setup(|app| {
            let quit = tauri::menu::MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&quit])?;

            app.notification()
                .builder()
                .title("Lumon")
                .body("Now on you tray!!!")
                .show()
                .unwrap();

            #[cfg(desktop)]
            {
                let autostart_manager = app.autolaunch();
                let _ = autostart_manager.enable();
                println!(
                    "registered for autostart? {}",
                    autostart_manager.is_enabled().unwrap()
                );
                let _ = autostart_manager.disable();
            }
            TrayIconBuilder::new()
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .tooltip("Welcome to Lumon")
                .show_menu_on_left_click(false)
                .on_tray_icon_event(move |tray, event| {
                    let app = tray.app_handle();
                    tauri_plugin_positioner::on_tray_event(app, &event);
                    if let Some(win) = app.get_webview_window("main") {
                        let _ = win.move_window_constrained(Position::TrayCenter);
                        match event {
                            TrayIconEvent::Click {
                                button: MouseButton::Left,
                                button_state: MouseButtonState::Up,
                                ..
                            } => {
                                if win.is_visible().unwrap() {
                                    let _ = win.reload();
                                    let _ = win.hide();
                                } else {
                                    let _ = win.show();
                                    // let _ = win.set_focus();
                                }
                            }
                            _ => {}
                        }
                    }
                })
                .build(app)?;
            Ok(())
        })
        .on_menu_event(|app, e| {
            if e.id.as_ref() == "quit" {
                app.exit(0);
            }
        })
        .on_window_event(|window, event| match event {
            WindowEvent::Focused(focused) => {
                if !focused {
                    window.hide().unwrap();
                }
            }
            _ => {}
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
