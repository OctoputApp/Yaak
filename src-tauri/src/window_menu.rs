/*
use crate::is_dev;
use tauri::menu::{AboutMetadata, Menu, MenuBuilder, MenuItem, Submenu, SubmenuBuilder};
use tauri::{AppHandle, Wry};

pub fn os_default(app_handle: &AppHandle, #[allow(unused)] app_name: &str) -> Menu<Wry> {
    let mut menu = MenuBuilder::new(app_handle);
    #[cfg(target_os = "macos")]
    {
        menu = menu.item(SubmenuBuilder::new(
            app_handle,
            app_name,).item(
            Menu::new(app_handle)
                .add_native_item(MenuItem::About(
                    app_name.to_string(),
                    AboutMetadata::default(),
                ))
                .add_native_item(MenuItem::Separator)
                .add_item(MenuItem::new(
                    "toggle_settings".to_string(),
                    "Settings",
                    true,
                    Some("CmdOrCtrl+,"),
                ))
                .add_native_item(MenuItem::Separator)
                .add_native_item(MenuItem::Services)
                .add_native_item(MenuItem::Separator)
                .add_native_item(MenuItem::Hide)
                .add_native_item(MenuItem::HideOthers)
                .add_native_item(MenuItem::ShowAll)
                .add_native_item(MenuItem::Separator)
                .add_native_item(MenuItem::Quit),
            true,
        ));
    }

    let mut file_menu = Menu::new(app_handle);
    file_menu = file_menu.add_native_item(MenuItem::CloseWindow);
    #[cfg(not(target_os = "macos"))]
    {
        file_menu = file_menu.add_native_item(MenuItem::Quit);
    }
    menu = menu.add_submenu(Submenu::new("File", file_menu, true));

    #[cfg(not(target_os = "linux"))]
    let mut edit_menu = Menu::new(app_handle);
    #[cfg(target_os = "macos")]
    {
        edit_menu = edit_menu.add_native_item(MenuItem::Undo);
        edit_menu = edit_menu.add_native_item(MenuItem::Redo);
        edit_menu = edit_menu.add_native_item(MenuItem::Separator);
    }
    #[cfg(not(target_os = "linux"))]
    {
        edit_menu = edit_menu.add_native_item(MenuItem::Cut);
        edit_menu = edit_menu.add_native_item(MenuItem::Copy);
        edit_menu = edit_menu.add_native_item(MenuItem::Paste);
    }
    #[cfg(target_os = "macos")]
    {
        edit_menu = edit_menu.add_native_item(MenuItem::SelectAll);
    }
    #[cfg(not(target_os = "linux"))]
    {
        menu = menu.add_submenu(Submenu::new("Edit", edit_menu, true));
    }
    let mut view_menu = Menu::new(app_handle);
    #[cfg(target_os = "macos")]
    {
        view_menu = view_menu
            .add_native_item(MenuItem::EnterFullScreen)
            .add_native_item(MenuItem::Separator);
    }
    view_menu = view_menu
        .add_item(MenuItem::new(
            "zoom_reset".to_string(),
            "Zoom to Actual Size",
            true,
            "CmdOrCtrl+0",
        ))
        .add_item(MenuItem::new(
            "zoom_in".to_string(),
            "Zoom In",
            true,
            "CmdOrCtrl+Plus",
        ))
        .add_item(MenuItem::new(
            "zoom_out".to_string(),
            "Zoom Out",
            true,
            "CmdOrCtrl+-",
        ));
    // .add_native_item(MenuItem::Separator)
    // .add_item(
    //     CustomMenuItem::new("toggle_sidebar".to_string(), "Toggle Sidebar")
    //         .accelerator("CmdOrCtrl+b"),
    // )
    // .add_item(
    //     CustomMenuItem::new("focus_sidebar".to_string(), "Focus Sidebar")
    //         .accelerator("CmdOrCtrl+1"),
    // )
    // .add_item(
    //     CustomMenuItem::new("toggle_settings".to_string(), "Toggle Settings")
    //         .accelerator("CmdOrCtrl+,"),
    // )
    // .add_item(
    //     CustomMenuItem::new("focus_url".to_string(), "Focus URL").accelerator("CmdOrCtrl+l"),
    // );
    menu = menu.add_submenu(Submenu::new("View", view_menu, true));

    let mut window_menu = Menu::new(app_handle);
    window_menu = window_menu.add_native_item(MenuItem::Minimize);
    #[cfg(target_os = "macos")]
    {
        window_menu = window_menu.add_native_item(MenuItem::Zoom);
        window_menu = window_menu.add_native_item(MenuItem::Separator);
    }
    window_menu = window_menu.add_native_item(MenuItem::CloseWindow);
    menu = menu.add_submenu(Submenu::new("Window", window_menu, true));

    // menu = menu.add_submenu(Submenu::new(
    //     "Workspace",
    //     Menu::new()
    //         .add_item(
    //             CustomMenuItem::new("send_request".to_string(), "Send Request")
    //                 .accelerator("CmdOrCtrl+r"),
    //         )
    //         .add_item(
    //             CustomMenuItem::new("new_request".to_string(), "New Request")
    //                 .accelerator("CmdOrCtrl+n"),
    //         )
    //         .add_item(
    //             CustomMenuItem::new("duplicate_request".to_string(), "Duplicate Request")
    //                 .accelerator("CmdOrCtrl+d"),
    //         ),
    // ));

    if is_dev() {
        menu = menu.add_submenu(Submenu::new(
            "Developer",
            Menu::new(app_handle)
                .add_item(MenuItem::new(
                    "refresh".to_string(),
                    "Refresh",
                    true,
                    "CmdOrCtrl + Shift + r",
                ))
                .add_item(MenuItem::new(
                    "toggle_devtools".to_string(),
                    "Open Devtools",
                    true,
                    "CmdOrCtrl + Option + i",
                )),
            true,
        ));
    }

    menu
}
*/
