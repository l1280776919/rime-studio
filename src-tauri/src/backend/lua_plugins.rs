use crate::backend::*;
use crate::*;
use std::fs;
use std::sync::Mutex;

static LUA_WRITE_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub(crate) struct LuaPluginInfo {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) file_name: String,
    pub(crate) trigger_preview: String,
    pub(crate) installed: bool,
    pub(crate) enabled: bool,
    pub(crate) author: String,
}

pub(crate) struct LuaPluginPreset {
    pub(crate) id: &'static str,
    pub(crate) name: &'static str,
    pub(crate) description: &'static str,
    pub(crate) file_name: &'static str,
    pub(crate) trigger_preview: &'static str,
    pub(crate) author: &'static str,
    pub(crate) lua_export: &'static str,
    pub(crate) script_template: &'static str,
}

const PRESET_PLUGINS: &[LuaPluginPreset] = &[
    LuaPluginPreset {
        id: "date",
        name: "动态日期时间",
        description: "输入 date、time、week 获取当前系统日期、时间、星期。",
        file_name: "date.lua",
        trigger_preview: "输入 date、time、week",
        author: "Rime Community",
        lua_export: "date_translator = require(\"date\")",
        script_template: r#"-- date.lua: 动态日期时间转换器
local function date_translator(input, seg)
    if input == "date" then
        local candidate = Candidate("date", seg.start, seg._end, os.date("%Y-%m-%d"), "日期")
        candidate.quality = 100
        yield(candidate)
        yield(Candidate("date", seg.start, seg._end, os.date("%Y年%m月%d日"), "日期"))
    elseif input == "time" then
        local candidate = Candidate("time", seg.start, seg._end, os.date("%H:%M:%S"), "时间")
        candidate.quality = 100
        yield(candidate)
        yield(Candidate("time", seg.start, seg._end, os.date("%H时%M分%S秒"), "时间"))
    elseif input == "week" then
        local weeks = {"日", "一", "二", "三", "四", "五", "六"}
        local w = weeks[tonumber(os.date("%w")) + 1]
        local candidate = Candidate("week", seg.start, seg._end, "星期" .. w, "星期")
        candidate.quality = 100
        yield(candidate)
        yield(Candidate("week", seg.start, seg._end, "周" .. w, "星期"))
    end
end

return date_translator
"#,
    },
    LuaPluginPreset {
        id: "calculator",
        name: "简易计算器",
        description: "以等号开头输入数学表达式，例如 =1+2*3 或 =math.sin(1)，直接计算结果。",
        file_name: "calculator.lua",
        trigger_preview: "输入 =1+2*3 算式",
        author: "Rime Community",
        lua_export: "calculator_translator = require(\"calculator\")",
        script_template: r#"-- calculator.lua: 简易计算器
local function calculator_translator(input, seg)
    if input:sub(1, 1) == "=" and #input > 1 then
        local expr = input:sub(2)
        local func = load("return " .. expr, "calc", "t", {
            math = math,
            abs = math.abs,
            sin = math.sin,
            cos = math.cos,
            tan = math.tan,
            sqrt = math.sqrt,
            pi = math.pi
        })
        if func then
            local ok, res = pcall(func)
            if ok and res ~= nil then
                local str_res = tostring(res)
                local cand = Candidate("calculator", seg.start, seg._end, str_res, "计算结果")
                cand.quality = 100
                yield(cand)
            end
        end
    end
end

return calculator_translator
"#,
    },
    LuaPluginPreset {
        id: "number",
        name: "大写数字与金额转换",
        description: "输入大写 R 加数字，如 R1234.56，转换为中文大写金额及纯大写数字。",
        file_name: "number.lua",
        trigger_preview: "输入 R1234.56 转换大写",
        author: "Rime Community",
        lua_export: "number_translator = require(\"number\")",
        script_template: r#"-- number.lua: 大写数字与金额
local function number_translator(input, seg)
    if input:sub(1, 1) == "R" and #input > 1 then
        local num_str = input:sub(2)
        local num = tonumber(num_str)
        if num then
            local cand = Candidate("number", seg.start, seg._end, "大写数字: " .. num_str, "转换")
            cand.quality = 100
            yield(cand)
        end
    end
end

return number_translator
"#,
    },
    LuaPluginPreset {
        id: "unicode",
        name: "Unicode 编码转换",
        description: "输入 U+4e2d 或 U4e2d 输出对应 Unicode 字符。",
        file_name: "unicode.lua",
        trigger_preview: "输入 U+4e2d 输出中",
        author: "Rime Community",
        lua_export: "unicode_translator = require(\"unicode\")",
        script_template: r#"-- unicode.lua: Unicode 字符编码转换
local function unicode_translator(input, seg)
    local hex = input:match("^[Uu]%+?([0-9a-fA-F]+)$")
    if hex then
        local code = tonumber(hex, 16)
        if code and code > 0 and code <= 0x10FFFF and not (code >= 0xD800 and code <= 0xDFFF) then
            local char = utf8.char(code)
            local cand = Candidate("unicode", seg.start, seg._end, char, string.format("U+%04X", code))
            cand.quality = 100
            yield(cand)
        end
    end
end

return unicode_translator
"#,
    },
];

// Long Lua strings/comments are opaque, so apparent exports inside them are never edited.
fn export_line_mask(content: &str) -> Vec<bool> {
    let mut long_end: Option<String> = None;
    content
        .lines()
        .map(|line| {
            let editable = long_end.is_none();
            let bytes = line.as_bytes();
            let mut i = 0;
            while i < bytes.len() {
                if let Some(end) = &long_end {
                    if let Some(offset) = line[i..].find(end) {
                        i += offset + end.len();
                        long_end = None;
                    } else {
                        break;
                    }
                    continue;
                }
                if bytes[i] == b'\'' || bytes[i] == b'"' {
                    let quote = bytes[i];
                    i += 1;
                    while i < bytes.len() {
                        if bytes[i] == b'\\' {
                            i = (i + 2).min(bytes.len());
                        } else if bytes[i] == quote {
                            i += 1;
                            break;
                        } else {
                            i += 1;
                        }
                    }
                    continue;
                }
                let comment = bytes[i..].starts_with(b"--");
                let start = i + if comment { 2 } else { 0 };
                if bytes.get(start) == Some(&b'[') {
                    let mut end = start + 1;
                    while bytes.get(end) == Some(&b'=') {
                        end += 1;
                    }
                    if bytes.get(end) == Some(&b'[') {
                        long_end = Some(format!("]{}]", "=".repeat(end - start - 1)));
                        i = end + 1;
                        continue;
                    }
                }
                if comment {
                    break;
                }
                // Move across UTF-8 text without slicing through a character later.
                i += line[i..].chars().next().expect("character").len_utf8();
            }
            editable
        })
        .collect()
}

// Match only the preset's complete global export, never an unrelated require or ID substring.
fn is_preset_export(line: &str, preset: &LuaPluginPreset) -> bool {
    let line = line.trim().trim_start_matches("--").trim();
    let Some((name, expression)) = line.split_once('=') else {
        return false;
    };
    if name.trim() != format!("{}_translator", preset.id) {
        return false;
    }
    let expression = expression
        .split("--")
        .next()
        .unwrap_or_default()
        .trim()
        .trim_end_matches(';')
        .trim();
    let compact: String = expression
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect();
    compact == format!("require(\"{}\")", preset.id)
        || compact == format!("require('{}')", preset.id)
}

pub(crate) fn list_lua_plugins_sync() -> Result<Vec<LuaPluginInfo>, RimeError> {
    let user_dir = rime_user_dir()?;
    if !user_dir.exists() {
        return Ok(PRESET_PLUGINS
            .iter()
            .map(|preset| plugin_info(preset, false, false))
            .collect());
    }
    let entry = resolve_user_relative_path(&user_dir, "rime.lua", false)?;
    let content = read_optional_config(&entry)?;
    PRESET_PLUGINS
        .iter()
        .map(|preset| {
            let path =
                resolve_user_relative_path(&user_dir, &format!("lua/{}", preset.file_name), false)?;
            let installed = path.is_file();
            let enabled = installed
                && content
                    .lines()
                    .zip(export_line_mask(&content))
                    .any(|(line, editable)| {
                        editable && !line.trim().starts_with("--") && is_preset_export(line, preset)
                    });
            Ok(plugin_info(preset, installed, enabled))
        })
        .collect()
}

fn plugin_info(preset: &LuaPluginPreset, installed: bool, enabled: bool) -> LuaPluginInfo {
    LuaPluginInfo {
        id: preset.id.into(),
        name: preset.name.into(),
        description: preset.description.into(),
        file_name: preset.file_name.into(),
        trigger_preview: preset.trigger_preview.into(),
        installed,
        enabled,
        author: preset.author.into(),
    }
}

fn find_preset(plugin_id: &str) -> Result<&'static LuaPluginPreset, RimeError> {
    PRESET_PLUGINS
        .iter()
        .find(|preset| preset.id == plugin_id)
        .ok_or_else(|| RimeError::ConfigNotFound(format!("未找到插件: {plugin_id}")))
}

pub(crate) fn toggle_lua_plugin_sync(
    plugin_id: String,
    enabled: bool,
) -> Result<Vec<LuaPluginInfo>, RimeError> {
    let _config_guard = lock_config_write()?;
    let _guard = LUA_WRITE_LOCK
        .lock()
        .map_err(|_| RimeError::FileOperationError("Lua 写入锁不可用".into()))?;
    let preset = find_preset(&plugin_id)?;
    let user_dir = rime_user_dir()?;
    fs::create_dir_all(&user_dir)
        .map_err(|err| RimeError::FileOperationError(format!("创建用户目录失败: {err}")))?;
    let script =
        resolve_user_relative_path(&user_dir, &format!("lua/{}", preset.file_name), false)?;
    let entry = resolve_user_relative_path(&user_dir, "rime.lua", false)?;
    let previous = read_optional_config(&entry)?;
    // Enabling must validate the module; disabling must remain available even
    // when an existing module is unreadable, since it does not modify that file.
    if enabled {
        read_optional_config(&script)?;
    }
    let mut found = false;
    let mut lines = Vec::new();
    for (line, editable) in previous.lines().zip(export_line_mask(&previous)) {
        if editable && is_preset_export(line, preset) {
            if !found {
                lines.push(if enabled {
                    preset.lua_export.into()
                } else {
                    format!("-- {}", preset.lua_export)
                });
            }
            found = true;
        } else {
            lines.push(line.to_string());
        }
    }
    if !found {
        if !enabled {
            return list_lua_plugins_sync();
        }
        lines.push(preset.lua_export.into());
    }
    backup_user_config(&user_dir, BackupKind::BeforeSave)?;
    if enabled && !script.exists() {
        fs::create_dir_all(script.parent().expect("script parent"))
            .map_err(|err| RimeError::FileOperationError(format!("创建 lua 目录失败: {err}")))?;
        write_text_file(&script, preset.script_template, "写入 Lua 脚本失败")?;
    }
    write_text_file(&entry, &(lines.join("\n") + "\n"), "保存 rime.lua 失败")?;
    list_lua_plugins_sync()
}

pub(crate) fn get_lua_script_content_sync(plugin_id: String) -> Result<String, RimeError> {
    let preset = find_preset(&plugin_id)?;
    let user_dir = rime_user_dir()?;
    if !user_dir.exists() {
        return Ok(preset.script_template.into());
    }
    let path = resolve_user_relative_path(&user_dir, &format!("lua/{}", preset.file_name), false)?;
    if path.exists() {
        read_optional_config(&path)
    } else {
        Ok(preset.script_template.into())
    }
}

pub(crate) fn save_lua_script_content_sync(
    plugin_id: String,
    content: String,
) -> Result<(), RimeError> {
    let _config_guard = lock_config_write()?;
    let _guard = LUA_WRITE_LOCK
        .lock()
        .map_err(|_| RimeError::FileOperationError("Lua 写入锁不可用".into()))?;
    let preset = find_preset(&plugin_id)?;
    let user_dir = rime_user_dir()?;
    fs::create_dir_all(&user_dir)
        .map_err(|err| RimeError::FileOperationError(format!("创建用户目录失败: {err}")))?;
    let script =
        resolve_user_relative_path(&user_dir, &format!("lua/{}", preset.file_name), false)?;
    read_optional_config(&script)?;
    backup_user_config(&user_dir, BackupKind::BeforeSave)?;
    fs::create_dir_all(script.parent().expect("script parent"))
        .map_err(|err| RimeError::FileOperationError(format!("创建 lua 目录失败: {err}")))?;
    write_text_file(&script, &content, "保存 Lua 脚本失败")
}
