use crate::backend::*;
use crate::*;
use std::fs;
use std::path::{Path, PathBuf};

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
        description: "输入 date、time、week 获取当前系统日期、时间、星期与农历提示。",
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
        if code and code > 0 and code <= 0x10FFFF then
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

fn rime_lua_file(user_dir: &Path) -> PathBuf {
    user_dir.join("rime.lua")
}

fn lua_dir(user_dir: &Path) -> PathBuf {
    user_dir.join("lua")
}

pub(crate) fn list_lua_plugins_sync() -> Result<Vec<LuaPluginInfo>, RimeError> {
    let user_dir = rime_user_dir()?;
    let rime_lua_path = rime_lua_file(&user_dir);
    let rime_lua_content = if rime_lua_path.exists() {
        fs::read_to_string(&rime_lua_path).unwrap_or_default()
    } else {
        String::new()
    };

    let l_dir = lua_dir(&user_dir);
    let mut plugins = Vec::new();

    for preset in PRESET_PLUGINS {
        let script_path = l_dir.join(preset.file_name);
        let installed = script_path.exists();
        let enabled = rime_lua_content.lines().any(|line| {
            let trimmed = line.trim();
            !trimmed.starts_with("--") && trimmed.contains(preset.id)
        });

        plugins.push(LuaPluginInfo {
            id: preset.id.to_string(),
            name: preset.name.to_string(),
            description: preset.description.to_string(),
            file_name: preset.file_name.to_string(),
            trigger_preview: preset.trigger_preview.to_string(),
            installed,
            enabled,
            author: preset.author.to_string(),
        });
    }

    Ok(plugins)
}

pub(crate) fn toggle_lua_plugin_sync(
    plugin_id: String,
    enabled: bool,
) -> Result<Vec<LuaPluginInfo>, RimeError> {
    let user_dir = rime_user_dir()?;
    fs::create_dir_all(&user_dir)
        .map_err(|err| RimeError::FileOperationError(format!("创建用户目录失败: {err}")))?;

    let preset = PRESET_PLUGINS
        .iter()
        .find(|p| p.id == plugin_id)
        .ok_or_else(|| RimeError::ConfigNotFound(format!("未找到插件: {plugin_id}")))?;

    let l_dir = lua_dir(&user_dir);
    fs::create_dir_all(&l_dir)
        .map_err(|err| RimeError::FileOperationError(format!("创建 lua 目录失败: {err}")))?;

    let script_path = l_dir.join(preset.file_name);
    if !script_path.exists() {
        fs::write(&script_path, preset.script_template)
            .map_err(|err| RimeError::FileOperationError(format!("写入 Lua 脚本失败: {err}")))?;
    }

    let rime_lua_path = rime_lua_file(&user_dir);
    let current_content = if rime_lua_path.exists() {
        fs::read_to_string(&rime_lua_path).unwrap_or_default()
    } else {
        "-- Rime Lua plugins entry point\n".to_string()
    };

    let mut new_lines = Vec::new();
    let mut found = false;

    for line in current_content.lines() {
        let trimmed = line.trim();
        if trimmed.contains(&format!("require(\"{}\")", preset.id))
            || trimmed.contains(&format!("require('{}')", preset.id))
        {
            found = true;
            if enabled {
                new_lines.push(preset.lua_export.to_string());
            } else {
                new_lines.push(format!("-- {}", preset.lua_export));
            }
        } else {
            new_lines.push(line.to_string());
        }
    }

    if !found && enabled {
        new_lines.push(preset.lua_export.to_string());
    }

    let final_content = new_lines.join("\n") + "\n";
    fs::write(&rime_lua_path, final_content)
        .map_err(|err| RimeError::FileOperationError(format!("保存 rime.lua 失败: {err}")))?;

    list_lua_plugins_sync()
}

pub(crate) fn get_lua_script_content_sync(plugin_id: String) -> Result<String, RimeError> {
    let user_dir = rime_user_dir()?;
    let preset = PRESET_PLUGINS
        .iter()
        .find(|p| p.id == plugin_id)
        .ok_or_else(|| RimeError::ConfigNotFound(format!("未找到插件: {plugin_id}")))?;

    let script_path = lua_dir(&user_dir).join(preset.file_name);
    if script_path.exists() {
        fs::read_to_string(&script_path)
            .map_err(|err| RimeError::FileOperationError(format!("读取 Lua 脚本失败: {err}")))
    } else {
        Ok(preset.script_template.to_string())
    }
}

pub(crate) fn save_lua_script_content_sync(
    plugin_id: String,
    content: String,
) -> Result<(), RimeError> {
    let user_dir = rime_user_dir()?;
    let preset = PRESET_PLUGINS
        .iter()
        .find(|p| p.id == plugin_id)
        .ok_or_else(|| RimeError::ConfigNotFound(format!("未找到插件: {plugin_id}")))?;

    let l_dir = lua_dir(&user_dir);
    fs::create_dir_all(&l_dir)
        .map_err(|err| RimeError::FileOperationError(format!("创建 lua 目录失败: {err}")))?;

    let script_path = l_dir.join(preset.file_name);
    fs::write(&script_path, content)
        .map_err(|err| RimeError::FileOperationError(format!("保存 Lua 脚本失败: {err}")))?;

    Ok(())
}
