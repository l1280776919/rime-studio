use serde::{Deserialize, Serialize};
use serde_yaml::{Mapping, Value};
use std::{
    env,
    ffi::OsStr,
    fs,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::{self, Command},
    time::{Instant, SystemTime, UNIX_EPOCH},
};
use tauri::Emitter;

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

type DictionaryEntry = (String, String, i32);
const AUTO_BACKUP_KEEP_LIMIT: usize = 30;
const MAX_DICTIONARY_DOWNLOAD_BYTES: usize = 64 * 1024 * 1024;
const MAX_LMDG_DOWNLOAD_BYTES: usize = 256 * 1024 * 1024;
const MAX_LMDG_GRAMMAR_BYTES: usize = 512 * 1024 * 1024;

const SOGOU_BIN_PINYIN: &[&str] = &[
    "a", "ai", "an", "ang", "ao", "ba", "bai", "ban", "bang", "bao", "bei", "ben", "beng", "bi",
    "bian", "biao", "bie", "bin", "bing", "bo", "bu", "ca", "cai", "can", "cang", "cao", "ce",
    "cen", "ceng", "cha", "chai", "chan", "chang", "chao", "che", "chen", "cheng", "chi", "chong",
    "chou", "chu", "chua", "chuai", "chuan", "chuang", "chui", "chun", "chuo", "ci", "cong", "cou",
    "cu", "cuan", "cui", "cun", "cuo", "da", "dai", "dan", "dang", "dao", "de", "dei", "den",
    "deng", "di", "dia", "dian", "diao", "die", "ding", "diu", "dong", "dou", "du", "duan", "dui",
    "dun", "duo", "e", "ei", "en", "eng", "er", "fa", "fan", "fang", "fei", "fen", "feng", "fiao",
    "fo", "fou", "fu", "ga", "gai", "gan", "gang", "gao", "ge", "gei", "gen", "geng", "gong",
    "gou", "gu", "gua", "guai", "guan", "guang", "gui", "gun", "guo", "ha", "hai", "han", "hang",
    "hao", "he", "hei", "hen", "heng", "hong", "hou", "hu", "hua", "huai", "huan", "huang", "hui",
    "hun", "huo", "ji", "jia", "jian", "jiang", "jiao", "jie", "jin", "jing", "jiong", "jiu", "ju",
    "juan", "jue", "jun", "ka", "kai", "kan", "kang", "kao", "ke", "kei", "ken", "keng", "kong",
    "kou", "ku", "kua", "kuai", "kuan", "kuang", "kui", "kun", "kuo", "la", "lai", "lan", "lang",
    "lao", "le", "lei", "leng", "li", "lia", "lian", "liang", "liao", "lie", "lin", "ling", "liu",
    "lo", "long", "lou", "lu", "luan", "lve", "lun", "luo", "lv", "ma", "mai", "man", "mang",
    "mao", "me", "mei", "men", "meng", "mi", "mian", "miao", "mie", "min", "ming", "miu", "mo",
    "mou", "mu", "na", "nai", "nan", "nang", "nao", "ne", "nei", "nen", "neng", "ni", "nian",
    "niang", "niao", "nie", "nin", "ning", "niu", "nong", "nou", "nu", "nuan", "nve", "nun", "nuo",
    "nv", "o", "ou", "pa", "pai", "pan", "pang", "pao", "pei", "pen", "peng", "pi", "pian", "piao",
    "pie", "pin", "ping", "po", "pou", "pu", "qi", "qia", "qian", "qiang", "qiao", "qie", "qin",
    "qing", "qiong", "qiu", "qu", "quan", "que", "qun", "ran", "rang", "rao", "re", "ren", "reng",
    "ri", "rong", "rou", "ru", "rua", "ruan", "rui", "run", "ruo", "sa", "sai", "san", "sang",
    "sao", "se", "sen", "seng", "sha", "shai", "shan", "shang", "shao", "she", "shei", "shen",
    "sheng", "shi", "shou", "shu", "shua", "shuai", "shuan", "shuang", "shui", "shun", "shuo",
    "si", "song", "sou", "su", "suan", "sui", "sun", "suo", "ta", "tai", "tan", "tang", "tao",
    "te", "tei", "teng", "ti", "tian", "tiao", "tie", "ting", "tong", "tou", "tu", "tuan", "tui",
    "tun", "tuo", "wa", "wai", "wan", "wang", "wei", "wen", "weng", "wo", "wu", "xi", "xia",
    "xian", "xiang", "xiao", "xie", "xin", "xing", "xiong", "xiu", "xu", "xuan", "xue", "xun",
    "ya", "yan", "yang", "yao", "ye", "yi", "yin", "ying", "yo", "yong", "you", "yu", "yuan",
    "yue", "yun", "za", "zai", "zan", "zang", "zao", "ze", "zei", "zen", "zeng", "zha", "zhai",
    "zhan", "zhang", "zhao", "zhe", "zhei", "zhen", "zheng", "zhi", "zhong", "zhou", "zhu", "zhua",
    "zhuai", "zhuan", "zhuang", "zhui", "zhun", "zhuo", "zi", "zong", "zou", "zu", "zuan", "zui",
    "zun", "zuo",
];

#[derive(Debug, Serialize)]
struct FileStatus {
    name: String,
    path: String,
    exists: bool,
    size: Option<u64>,
    modified: Option<u64>,
}

#[derive(Debug, Serialize)]
struct DictHealth {
    entries: usize,
    duplicate_exact_lines: usize,
    long_low_weight_entries: usize,
}

#[derive(Debug, Serialize)]
struct RimeEnvironment {
    user_dir: String,
    build_dir: String,
    deployer_path: Option<String>,
    plum_dir: String,
    git_available: bool,
    bash_available: bool,
    git_path: Option<String>,
    bash_path: Option<String>,
    active_schema: Option<String>,
    page_size: Option<u32>,
    theme_name: Option<String>,
    font_point: Option<u32>,
    label_font_point: Option<u32>,
    custom_files: Vec<FileStatus>,
    sogou_health: Option<DictHealth>,
}

#[derive(Debug, Serialize)]
struct DeployResult {
    success: bool,
    message: String,
}

#[derive(Debug, Serialize)]
struct InstallResult {
    success: bool,
    recipe: String,
    backup_dir: Option<String>,
    log: String,
}

#[derive(Debug, Serialize)]
struct BackupEntry {
    name: String,
    path: String,
    kind: String,
    modified: Option<u64>,
    files: usize,
}

#[derive(Debug, Serialize)]
struct RestoreResult {
    restored_files: usize,
    safety_backup_dir: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct PhraseEntry {
    text: String,
    code: String,
    weight: i32,
}

#[derive(Debug, Serialize)]
struct DictInfo {
    name: String,
    path: String,
    entry_count: usize,
    size_bytes: u64,
    modified: Option<u64>,
}

#[derive(Debug, Serialize)]
struct DictionaryImportResult {
    name: String,
    reference: String,
    path: String,
    imported_entries: usize,
    skipped_entries: usize,
}

#[derive(Debug, Serialize)]
struct DictionaryPreviewEntry {
    text: String,
    code: String,
    weight: i32,
}

#[derive(Debug, Serialize)]
struct DictionaryImportPreview {
    name: String,
    reference: String,
    path: String,
    imported_entries: usize,
    skipped_entries: usize,
    sample_entries: Vec<DictionaryPreviewEntry>,
    will_overwrite: bool,
}

#[derive(Debug, Serialize)]
struct DictionaryExportResult {
    name: String,
    contents: String,
}

#[derive(Debug, Serialize)]
struct DictionaryCleanResult {
    name: String,
    path: String,
    removed_duplicate_lines: usize,
    entries_after: usize,
    backup_dir: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct AppearanceConfig {
    theme_name: String,
    font_point: u32,
    label_font_point: u32,
    page_size: u32,
    switch_key: String,
    horizontal: bool,
    inline_preedit: bool,
    candidate_format: String,
    corner_radius: u32,
    border_height: u32,
    border_width: u32,
    line_spacing: u32,
    spacing: u32,
    back_color: String,
    border_color: String,
    text_color: String,
    candidate_text_color: String,
    comment_text_color: String,
    hilited_text_color: String,
    hilited_back_color: String,
    hilited_candidate_text_color: String,
    hilited_candidate_back_color: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct QuickSettingsConfig {
    schema_id: String,
    page_size: u32,
    switch_key: String,
    paging_keys: String,
    navigation_keys: String,
    horizontal: bool,
    inline_preedit: bool,
}

#[derive(Debug, Serialize)]
struct ConfigHealthCheck {
    name: String,
    status: String,
    detail: String,
}

#[derive(Debug, Serialize)]
struct ConfigHealthReport {
    summary: String,
    checks: Vec<ConfigHealthCheck>,
}

#[derive(Debug, Serialize)]
struct ConfigPreviewFile {
    name: String,
    path: String,
    changed: bool,
    diff_lines: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ConfigPreview {
    files: Vec<ConfigPreviewFile>,
}

#[derive(Debug, Deserialize, Serialize)]
struct RimeIceSettings {
    emoji: bool,
    traditionalization: bool,
    ascii_punct: bool,
    full_shape: bool,
    search_single_char: bool,
    fuzzy_pinyin: bool,
    traditional_preset: String,
}

#[derive(Debug, Serialize)]
struct DictionaryReference {
    reference: String,
    path: Option<String>,
    exists: bool,
    entry_count: Option<usize>,
    size_bytes: Option<u64>,
}

#[derive(Debug, Serialize)]
struct DictionaryConfig {
    schema_id: Option<String>,
    schema_name: Option<String>,
    main_dictionary: Option<String>,
    main_dictionary_path: Option<String>,
    enabled: Vec<DictionaryReference>,
    available: Vec<DictInfo>,
    missing: Vec<DictionaryReference>,
}

#[derive(Debug, Clone, Serialize)]
struct OnlineDictionary {
    id: String,
    title: String,
    category: String,
    description: String,
    source: String,
    source_name: String,
    detail_url: String,
}

#[derive(Debug, Clone, Serialize)]
struct OnlineDictionaryCategory {
    id: String,
    title: String,
    description: String,
}

#[derive(Debug, Serialize)]
struct LmdgInstallResult {
    installed_count: usize,
    target_dir: String,
    source_url: String,
    message: String,
}

#[derive(Debug, Serialize)]
struct LmdgGrammarInstallResult {
    model_name: String,
    model_path: String,
    patch_path: String,
    source_url: String,
    message: String,
}

#[derive(Debug, Serialize)]
struct LmdgGrammarUninstallResult {
    model_name: String,
    model_path: String,
    patch_path: String,
    removed_model: bool,
    message: String,
}

#[derive(Debug, Clone, Serialize)]
struct DownloadProgressPayload {
    kind: String,
    stage: String,
    downloaded_bytes: u64,
    total_bytes: Option<u64>,
    percent: Option<f64>,
}

include!("backend/core.rs");
include!("backend/appearance.rs");
include!("backend/backup.rs");
include!("backend/phrases.rs");
include!("backend/dictionaries.rs");
include!("backend/downloads.rs");
include!("backend/system.rs");
include!("backend/settings.rs");
include!("backend/schemas.rs");
include!("backend/tests.rs");

async fn run_blocking<T, F>(task: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(task)
        .await
        .map_err(|err| format!("后台任务失败: {err}"))?
}

#[tauri::command]
async fn scan_rime_environment() -> Result<RimeEnvironment, String> {
    run_blocking(scan_rime_environment_sync).await
}

#[tauri::command]
async fn deploy_rime() -> Result<DeployResult, String> {
    run_blocking(deploy_rime_sync).await
}

#[tauri::command]
async fn install_rime_ice(recipe: Option<String>) -> Result<InstallResult, String> {
    run_blocking(move || install_rime_ice_sync(recipe)).await
}

#[tauri::command]
async fn get_appearance_config() -> Result<AppearanceConfig, String> {
    run_blocking(get_appearance_config_sync).await
}

#[tauri::command]
async fn get_quick_settings() -> Result<QuickSettingsConfig, String> {
    run_blocking(get_quick_settings_sync).await
}

#[tauri::command]
async fn save_quick_settings(config: QuickSettingsConfig) -> Result<QuickSettingsConfig, String> {
    run_blocking(move || save_quick_settings_sync(config)).await
}

#[tauri::command]
async fn preview_quick_settings(config: QuickSettingsConfig) -> Result<ConfigPreview, String> {
    run_blocking(move || preview_quick_settings_sync(config)).await
}

#[tauri::command]
async fn inspect_config_health() -> Result<ConfigHealthReport, String> {
    run_blocking(inspect_config_health_sync).await
}

#[tauri::command]
async fn repair_config_health() -> Result<ConfigHealthReport, String> {
    run_blocking(repair_config_health_sync).await
}

#[tauri::command]
async fn repair_config_health_item(name: String) -> Result<ConfigHealthReport, String> {
    run_blocking(move || repair_config_health_item_sync(name)).await
}

#[tauri::command]
async fn get_rime_ice_settings() -> Result<RimeIceSettings, String> {
    run_blocking(get_rime_ice_settings_sync).await
}

#[tauri::command]
async fn save_rime_ice_settings(settings: RimeIceSettings) -> Result<RimeIceSettings, String> {
    run_blocking(move || save_rime_ice_settings_sync(settings)).await
}

#[tauri::command]
async fn save_appearance_config(config: AppearanceConfig) -> Result<AppearanceConfig, String> {
    run_blocking(move || save_appearance_config_sync(config)).await
}

#[tauri::command]
async fn list_backups() -> Result<Vec<BackupEntry>, String> {
    run_blocking(list_backups_sync).await
}

#[tauri::command]
async fn create_backup() -> Result<BackupEntry, String> {
    run_blocking(create_backup_sync).await
}

#[tauri::command]
async fn open_rime_user_dir() -> Result<(), String> {
    run_blocking(open_rime_user_dir_sync).await
}

#[tauri::command]
async fn open_config_file(name: String) -> Result<(), String> {
    run_blocking(move || open_config_file_sync(name)).await
}

#[tauri::command]
async fn open_plum_dir() -> Result<(), String> {
    run_blocking(open_plum_dir_sync).await
}

#[tauri::command]
async fn open_backup_dir(backup_name: String) -> Result<(), String> {
    run_blocking(move || open_backup_dir_sync(backup_name)).await
}

#[tauri::command]
async fn restore_backup(backup_name: String) -> Result<RestoreResult, String> {
    run_blocking(move || restore_backup_sync(backup_name)).await
}

#[tauri::command]
async fn delete_backup(backup_name: String) -> Result<(), String> {
    run_blocking(move || delete_backup_sync(backup_name)).await
}

#[tauri::command]
async fn delete_dictionary(dict_name: String) -> Result<(), String> {
    run_blocking(move || delete_dictionary_sync(dict_name)).await
}

#[tauri::command]
async fn get_custom_phrases() -> Result<Vec<PhraseEntry>, String> {
    run_blocking(get_custom_phrases_sync).await
}

#[tauri::command]
async fn save_custom_phrases(phrases: Vec<PhraseEntry>) -> Result<(), String> {
    run_blocking(move || save_custom_phrases_sync(phrases)).await
}

#[tauri::command]
async fn list_dictionaries() -> Result<Vec<DictInfo>, String> {
    run_blocking(scan_dictionaries_sync_wrapper).await
}

#[tauri::command]
async fn get_dict_health(dict_name: String) -> Result<DictHealth, String> {
    run_blocking(move || get_dict_health_sync_wrapper(dict_name)).await
}

#[tauri::command]
async fn clean_dictionary_duplicates(dict_name: String) -> Result<DictionaryCleanResult, String> {
    run_blocking(move || clean_dictionary_duplicates_sync(dict_name)).await
}

#[tauri::command]
async fn get_dictionary_config() -> Result<DictionaryConfig, String> {
    run_blocking(read_dictionary_config_sync).await
}

#[tauri::command]
async fn add_dictionary_to_current_schema(reference: String) -> Result<DictionaryConfig, String> {
    run_blocking(move || add_dictionary_to_current_schema_sync(reference)).await
}

#[tauri::command]
async fn remove_dictionary_from_current_schema(
    reference: String,
) -> Result<DictionaryConfig, String> {
    run_blocking(move || remove_dictionary_from_current_schema_sync(reference)).await
}

#[tauri::command]
async fn save_dictionary_imports(imports: Vec<String>) -> Result<DictionaryConfig, String> {
    run_blocking(move || save_dictionary_imports_sync(imports)).await
}

#[tauri::command]
async fn list_online_dictionaries() -> Result<Vec<OnlineDictionary>, String> {
    run_blocking(list_online_dictionaries_sync).await
}

#[tauri::command]
async fn list_online_dictionary_categories() -> Result<Vec<OnlineDictionaryCategory>, String> {
    run_blocking(list_online_dictionary_categories_sync).await
}

#[tauri::command]
async fn list_online_dictionaries_by_category(
    category_id: String,
) -> Result<Vec<OnlineDictionary>, String> {
    run_blocking(move || list_online_dictionaries_by_category_sync(category_id)).await
}

#[tauri::command]
async fn preview_online_dictionary_import(id: String) -> Result<DictionaryImportPreview, String> {
    run_blocking(move || preview_online_dictionary_import_sync(id)).await
}

#[tauri::command]
async fn import_online_dictionary(id: String) -> Result<DictionaryImportResult, String> {
    run_blocking(move || import_online_dictionary_sync(id)).await
}

#[tauri::command]
async fn preview_dictionary_url_import(
    url: String,
    source_name: Option<String>,
) -> Result<DictionaryImportPreview, String> {
    run_blocking(move || preview_dictionary_url_import_sync(url, source_name)).await
}

#[tauri::command]
async fn import_dictionary_url(
    url: String,
    source_name: Option<String>,
) -> Result<DictionaryImportResult, String> {
    run_blocking(move || import_dictionary_url_sync(url, source_name)).await
}

#[tauri::command]
async fn install_lmdg_dicts(window: tauri::Window) -> Result<LmdgInstallResult, String> {
    run_blocking(move || {
        install_lmdg_dicts_sync_with_progress(|downloaded, total| {
            emit_download_progress(&window, "dicts", "下载万象词库包", downloaded, total);
        })
    })
    .await
}

#[tauri::command]
async fn install_lmdg_grammar(window: tauri::Window) -> Result<LmdgGrammarInstallResult, String> {
    run_blocking(move || {
        install_lmdg_grammar_sync_with_progress(|downloaded, total| {
            emit_download_progress(&window, "grammar", "下载万象语言模型", downloaded, total);
        })
    })
    .await
}

#[tauri::command]
async fn uninstall_lmdg_grammar() -> Result<LmdgGrammarUninstallResult, String> {
    run_blocking(uninstall_lmdg_grammar_sync).await
}

#[tauri::command]
async fn preview_dictionary_import(
    source_name: String,
    data: Vec<u8>,
) -> Result<DictionaryImportPreview, String> {
    run_blocking(move || preview_dictionary_import_sync(source_name, data)).await
}

#[tauri::command]
async fn import_dictionary(
    source_name: String,
    data: Vec<u8>,
) -> Result<DictionaryImportResult, String> {
    run_blocking(move || import_dictionary_sync(source_name, data)).await
}

#[tauri::command]
async fn export_dictionary(dict_name: String) -> Result<DictionaryExportResult, String> {
    run_blocking(move || export_dictionary_sync(dict_name)).await
}

#[tauri::command]
async fn download_rime_installer() -> Result<RimeDownloadResult, String> {
    run_blocking(download_rime_installer_sync).await
}

#[tauri::command]
async fn download_git_installer() -> Result<RimeDownloadResult, String> {
    run_blocking(download_git_installer_sync).await
}

#[tauri::command]
async fn launch_rime_installer(path: String) -> Result<(), String> {
    run_blocking(move || launch_installer_sync(path)).await
}

#[tauri::command]
async fn launch_git_installer(path: String) -> Result<(), String> {
    run_blocking(move || launch_installer_sync(path)).await
}

#[tauri::command]
async fn list_schemas() -> Result<Vec<SchemaInfo>, String> {
    run_blocking(list_schemas_sync).await
}

#[tauri::command]
async fn copy_schema(schema_id: String) -> Result<String, String> {
    run_blocking(move || copy_schema_sync(schema_id)).await
}

#[tauri::command]
async fn set_active_schema(schema_id: String) -> Result<QuickSettingsConfig, String> {
    run_blocking(move || set_active_schema_sync(schema_id)).await
}

#[tauri::command]
async fn save_active_schema_list(schema_ids: Vec<String>) -> Result<QuickSettingsConfig, String> {
    run_blocking(move || save_active_schema_list_sync(schema_ids)).await
}

#[tauri::command]
async fn open_schema_file(path: String) -> Result<(), String> {
    run_blocking(move || open_schema_file_sync(path)).await
}

#[tauri::command]
async fn open_schema_dir(path: String) -> Result<(), String> {
    run_blocking(move || open_schema_dir_sync(path)).await
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            scan_rime_environment,
            deploy_rime,
            install_rime_ice,
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
            open_schema_dir
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
