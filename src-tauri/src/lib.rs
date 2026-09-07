pub(crate) mod types;
pub(crate) use types::*;

pub(crate) mod backend;

pub(crate) mod commands;
pub(crate) use commands::*;

use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .build(),
        )
        .setup(|app| {
            let show_item = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
            let deploy_item =
                MenuItem::with_id(app, "deploy", "重新部署 Rime", true, None::<&str>)?;
            let restart_item =
                MenuItem::with_id(app, "restart", "重启小狼毫服务", true, None::<&str>)?;
            let sync_item = MenuItem::with_id(app, "sync", "同步用户词库", true, None::<&str>)?;
            let dir_item = MenuItem::with_id(app, "open_dir", "打开配置目录", true, None::<&str>)?;
            let quit_item = MenuItem::with_id(app, "quit", "退出 Rime Studio", true, None::<&str>)?;

            let menu = Menu::with_items(
                app,
                &[
                    &show_item,
                    &deploy_item,
                    &restart_item,
                    &sync_item,
                    &dir_item,
                    &quit_item,
                ],
            )?;

            let mut builder = TrayIconBuilder::new()
                .menu(&menu)
                .show_menu_on_left_click(false)
                .tooltip("Rime Studio - 小狼毫配置工作台");

            if let Some(icon) = app.default_window_icon() {
                builder = builder.icon(icon.clone());
            }

            builder
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => {
                        if let Some(window) = app.get_webview_window("main") {
                            let _ = window.show();
                            let _ = window.unminimize();
                            let _ = window.set_focus();
                        }
                    }
                    "deploy" => {
                        let app_handle = app.clone();
                        tauri::async_runtime::spawn(async move {
                            let _ = crate::backend::system::deploy_rime_sync(Some(app_handle));
                        });
                    }
                    "restart" => {
                        tauri::async_runtime::spawn(async move {
                            let _ = crate::backend::system::restart_weasel_server_sync();
                        });
                    }
                    "sync" => {
                        tauri::async_runtime::spawn(async move {
                            let _ = crate::backend::system::sync_rime_sync();
                        });
                    }
                    "open_dir" => {
                        let _ = crate::backend::settings::open_rime_user_dir_sync();
                    }
                    "quit" => {
                        app.exit(0);
                    }
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        let app = tray.app_handle();
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.unminimize();
                                let _ = window.set_focus();
                            }
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            scan_rime_environment,
            scan_dictionary_health,
            deploy_rime,
            cancel_deploy,
            restart_weasel_server,
            get_sync_config,
            save_sync_config,
            sync_rime,
            list_userdb_entries,
            install_rime_ice,
            get_appearance_config,
            preview_appearance_config,
            list_system_fonts,
            get_quick_settings,
            save_quick_settings,
            preview_quick_settings,
            inspect_config_health,
            repair_config_health,
            repair_config_health_item,
            get_rime_ice_settings,
            save_rime_ice_settings,
            preview_rime_ice_settings,
            save_appearance_config,
            list_backups,
            create_backup,
            preview_backup,
            open_rime_user_dir,
            open_sync_dir,
            open_config_file,
            open_plum_dir,
            open_backup_dir,
            open_app_log_dir,
            restore_backup,
            delete_backup,
            delete_dictionary,
            get_custom_phrases,
            save_custom_phrases,
            list_dictionaries,
            get_dict_health,
            clean_dictionary_duplicates,
            get_dictionary_config,
            add_dictionary_to_current_schema,
            remove_dictionary_from_current_schema,
            save_dictionary_imports,
            list_online_dictionaries,
            list_online_dictionary_categories,
            list_online_dictionaries_by_category,
            preview_online_dictionary_import,
            import_online_dictionary,
            preview_dictionary_url_import,
            import_dictionary_url,
            install_lmdg_dicts,
            install_lmdg_grammar,
            uninstall_lmdg_grammar,
            preview_dictionary_import,
            import_dictionary,
            export_dictionary,
            check_app_update,
            download_app_update,
            download_rime_installer,
            download_git_installer,
            launch_rime_installer,
            launch_git_installer,
            list_schemas,
            list_community_schemas,
            copy_schema,
            set_active_schema,
            save_active_schema_list,
            open_schema_file,
            open_schema_dir,
            list_lua_plugins,
            toggle_lua_plugin,
            get_lua_script_content,
            save_lua_script_content,
            list_yaml_config_files,
            read_config_file_content,
            write_config_file_content
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
