use serde::{Deserialize, Serialize};

#[cfg(windows)]
pub(crate) use std::os::windows::process::CommandExt;

#[cfg(windows)]
pub(crate) const CREATE_NO_WINDOW: u32 = 0x08000000;

pub(crate) type DictionaryEntry = (String, String, i32);
pub(crate) const AUTO_BACKUP_KEEP_LIMIT: usize = 30;
pub(crate) const MAX_DICTIONARY_DOWNLOAD_BYTES: usize = 64 * 1024 * 1024;
pub(crate) const MAX_LMDG_DOWNLOAD_BYTES: usize = 256 * 1024 * 1024;
pub(crate) const MAX_LMDG_GRAMMAR_BYTES: usize = 512 * 1024 * 1024;

pub(crate) const SOGOU_BIN_PINYIN: &[&str] = &[
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

// ── Struct definitions ────────────────────────────

#[derive(Debug, Serialize)]
pub(crate) struct FileStatus {
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) exists: bool,
    pub(crate) size: Option<u64>,
    pub(crate) modified: Option<u64>,
}

#[derive(Debug, Serialize)]
pub(crate) struct DictHealth {
    pub(crate) entries: usize,
    pub(crate) duplicate_exact_lines: usize,
    pub(crate) long_low_weight_entries: usize,
}

#[derive(Debug, Serialize)]
pub(crate) struct RimeEnvironment {
    pub(crate) user_dir: String,
    pub(crate) build_dir: String,
    pub(crate) deployer_path: Option<String>,
    pub(crate) plum_dir: String,
    pub(crate) git_available: bool,
    pub(crate) bash_available: bool,
    pub(crate) git_path: Option<String>,
    pub(crate) bash_path: Option<String>,
    pub(crate) active_schema: Option<String>,
    pub(crate) page_size: Option<u32>,
    pub(crate) theme_name: Option<String>,
    pub(crate) font_point: Option<u32>,
    pub(crate) label_font_point: Option<u32>,
    pub(crate) custom_files: Vec<FileStatus>,
    pub(crate) sogou_health: Option<DictHealth>,
}

#[derive(Debug, Serialize)]
pub(crate) struct DeployResult {
    pub(crate) success: bool,
    pub(crate) message: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct InstallResult {
    pub(crate) success: bool,
    pub(crate) recipe: String,
    pub(crate) backup_dir: Option<String>,
    pub(crate) log: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct BackupEntry {
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) kind: String,
    pub(crate) modified: Option<u64>,
    pub(crate) files: usize,
}

#[derive(Debug, Serialize)]
pub(crate) struct RestoreResult {
    pub(crate) restored_files: usize,
    pub(crate) safety_backup_dir: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct PhraseEntry {
    pub(crate) text: String,
    pub(crate) code: String,
    pub(crate) weight: i32,
}

#[derive(Debug, Serialize)]
pub(crate) struct DictInfo {
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) entry_count: usize,
    pub(crate) size_bytes: u64,
    pub(crate) modified: Option<u64>,
}

#[derive(Debug, Serialize)]
pub(crate) struct DictionaryImportResult {
    pub(crate) name: String,
    pub(crate) reference: String,
    pub(crate) path: String,
    pub(crate) imported_entries: usize,
    pub(crate) skipped_entries: usize,
}

#[derive(Debug, Serialize)]
pub(crate) struct DictionaryPreviewEntry {
    pub(crate) text: String,
    pub(crate) code: String,
    pub(crate) weight: i32,
}

#[derive(Debug, Serialize)]
pub(crate) struct DictionaryImportPreview {
    pub(crate) name: String,
    pub(crate) reference: String,
    pub(crate) path: String,
    pub(crate) imported_entries: usize,
    pub(crate) skipped_entries: usize,
    pub(crate) sample_entries: Vec<DictionaryPreviewEntry>,
    pub(crate) will_overwrite: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct DictionaryExportResult {
    pub(crate) name: String,
    pub(crate) contents: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct DictionaryCleanResult {
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) removed_duplicate_lines: usize,
    pub(crate) entries_after: usize,
    pub(crate) backup_dir: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct AppearanceConfig {
    pub(crate) theme_name: String,
    pub(crate) font_point: u32,
    pub(crate) label_font_point: u32,
    pub(crate) page_size: u32,
    pub(crate) switch_key: String,
    pub(crate) horizontal: bool,
    pub(crate) inline_preedit: bool,
    pub(crate) candidate_format: String,
    pub(crate) corner_radius: u32,
    pub(crate) border_height: u32,
    pub(crate) border_width: u32,
    pub(crate) line_spacing: u32,
    pub(crate) spacing: u32,
    pub(crate) back_color: String,
    pub(crate) border_color: String,
    pub(crate) text_color: String,
    pub(crate) candidate_text_color: String,
    pub(crate) comment_text_color: String,
    pub(crate) hilited_text_color: String,
    pub(crate) hilited_back_color: String,
    pub(crate) hilited_candidate_text_color: String,
    pub(crate) hilited_candidate_back_color: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct QuickSettingsConfig {
    pub(crate) schema_id: String,
    pub(crate) page_size: u32,
    pub(crate) switch_key: String,
    pub(crate) paging_keys: String,
    pub(crate) navigation_keys: String,
    pub(crate) horizontal: bool,
    pub(crate) inline_preedit: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct ConfigHealthCheck {
    pub(crate) name: String,
    pub(crate) status: String,
    pub(crate) detail: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct ConfigHealthReport {
    pub(crate) summary: String,
    pub(crate) checks: Vec<ConfigHealthCheck>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ConfigPreviewFile {
    pub(crate) name: String,
    pub(crate) path: String,
    pub(crate) changed: bool,
    pub(crate) diff_lines: Vec<String>,
}

#[derive(Debug, Serialize)]
pub(crate) struct ConfigPreview {
    pub(crate) files: Vec<ConfigPreviewFile>,
}

#[derive(Debug, Deserialize, Serialize)]
pub(crate) struct RimeIceSettings {
    pub(crate) emoji: bool,
    pub(crate) traditionalization: bool,
    pub(crate) ascii_punct: bool,
    pub(crate) full_shape: bool,
    pub(crate) search_single_char: bool,
    pub(crate) fuzzy_pinyin: bool,
    pub(crate) traditional_preset: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct DictionaryReference {
    pub(crate) reference: String,
    pub(crate) path: Option<String>,
    pub(crate) exists: bool,
    pub(crate) entry_count: Option<usize>,
    pub(crate) size_bytes: Option<u64>,
}

#[derive(Debug, Serialize)]
pub(crate) struct DictionaryConfig {
    pub(crate) schema_id: Option<String>,
    pub(crate) schema_name: Option<String>,
    pub(crate) main_dictionary: Option<String>,
    pub(crate) main_dictionary_path: Option<String>,
    pub(crate) enabled: Vec<DictionaryReference>,
    pub(crate) available: Vec<DictInfo>,
    pub(crate) missing: Vec<DictionaryReference>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct OnlineDictionary {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) category: String,
    pub(crate) description: String,
    pub(crate) source: String,
    pub(crate) source_name: String,
    pub(crate) detail_url: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct OnlineDictionaryCategory {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) description: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct LmdgInstallResult {
    pub(crate) installed_count: usize,
    pub(crate) target_dir: String,
    pub(crate) source_url: String,
    pub(crate) message: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct LmdgGrammarInstallResult {
    pub(crate) model_name: String,
    pub(crate) model_path: String,
    pub(crate) patch_path: String,
    pub(crate) source_url: String,
    pub(crate) message: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct LmdgGrammarUninstallResult {
    pub(crate) model_name: String,
    pub(crate) model_path: String,
    pub(crate) patch_path: String,
    pub(crate) removed_model: bool,
    pub(crate) message: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DownloadProgressPayload {
    pub(crate) kind: String,
    pub(crate) stage: String,
    pub(crate) downloaded_bytes: u64,
    pub(crate) total_bytes: Option<u64>,
    pub(crate) percent: Option<f64>,
}

#[derive(Debug, Serialize)]
pub(crate) struct SchemaInfo {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) path: String,
    pub(crate) is_system: bool,
    pub(crate) is_active: bool,
    pub(crate) is_enabled: bool,
}

#[derive(Debug, Serialize)]
pub(crate) struct RimeDownloadResult {
    pub(crate) success: bool,
    pub(crate) installer_path: Option<String>,
    pub(crate) message: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct AppUpdateInfo {
    pub(crate) current_version: String,
    pub(crate) latest_version: Option<String>,
    pub(crate) release_name: Option<String>,
    pub(crate) release_notes: Option<String>,
    pub(crate) published_at: Option<String>,
    pub(crate) release_url: String,
    pub(crate) asset_name: Option<String>,
    pub(crate) asset_url: Option<String>,
    pub(crate) update_available: bool,
}
