pub(crate) mod types;
pub(crate) use types::*;

pub(crate) mod backend;

pub(crate) mod commands;
pub(crate) use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            scan_rime_environment,
            deploy_rime,
            install_rime_ice,
            check_app_update,
            download_app_update,
            get_appearance_config,
            get_quick_settings,
            save_quick_settings,
            preview_quick_settings,
            inspect_config_health,
            repair_config_health,
            repair_config_health_item,
            get_rime_ice_settings,
            save_rime_ice_settings,
            save_appearance_config,
            list_backups,
            create_backup,
            open_rime_user_dir,
            open_config_file,
            open_plum_dir,
            open_backup_dir,
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
            download_rime_installer,
            download_git_installer,
            launch_rime_installer,
            launch_git_installer,
            list_schemas,
            copy_schema,
            set_active_schema,
            save_active_schema_list,
            open_schema_file,
            open_schema_dir,
            list_yaml_config_files,
            read_config_file_content,
            write_config_file_content
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
