use crate::backend::*;
use crate::*;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::time::Instant;

pub(crate) fn url_encode_component(value: &str) -> String {
    let mut encoded = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push_str(&format!("%{byte:02X}"));
        }
    }
    encoded
}

pub(crate) fn sogou_dictionary_url(id: &str, title: &str) -> String {
    format!(
        "https://pinyin.sogou.com/d/dict/download_cell.php?id={}&name={}&f=detail",
        url_encode_component(id),
        url_encode_component(title)
    )
}

pub(crate) fn online_dictionary_categories() -> Vec<OnlineDictionaryCategory> {
    vec![
        OnlineDictionaryCategory {
            id: "167".to_string(),
            title: "城市地名".to_string(),
            description: "城市、道路、机构、景区和方言词库。".to_string(),
        },
        OnlineDictionaryCategory {
            id: "1".to_string(),
            title: "自然科学".to_string(),
            description: "天文、地理、生物、数学、物理等领域词。".to_string(),
        },
        OnlineDictionaryCategory {
            id: "76".to_string(),
            title: "社会科学".to_string(),
            description: "法律、经济、管理、教育、心理等常见术语。".to_string(),
        },
        OnlineDictionaryCategory {
            id: "96".to_string(),
            title: "工程应用".to_string(),
            description: "计算机、机械、建筑、电子、电力等专业词库。".to_string(),
        },
        OnlineDictionaryCategory {
            id: "127".to_string(),
            title: "农林渔畜".to_string(),
            description: "农业、林业、渔业、畜牧业词库。".to_string(),
        },
        OnlineDictionaryCategory {
            id: "132".to_string(),
            title: "医学".to_string(),
            description: "中医、西药、疾病、医疗器械和护理词库。".to_string(),
        },
        OnlineDictionaryCategory {
            id: "436".to_string(),
            title: "艺术".to_string(),
            description: "绘画、音乐、摄影、戏剧、书法等艺术词。".to_string(),
        },
        OnlineDictionaryCategory {
            id: "154".to_string(),
            title: "运动休闲".to_string(),
            description: "球类、棋牌、武术、垂钓、奥运等词库。".to_string(),
        },
        OnlineDictionaryCategory {
            id: "389".to_string(),
            title: "生活".to_string(),
            description: "理财、饮食、旅游、办公、手机、美容等词库。".to_string(),
        },
        OnlineDictionaryCategory {
            id: "367".to_string(),
            title: "娱乐".to_string(),
            description: "动漫、明星、汽车、影视、模型等词库。".to_string(),
        },
    ]
}

pub(crate) fn online_dictionary_catalog() -> Vec<OnlineDictionary> {
    vec![
        OnlineDictionary {
            id: "sogou_trending".to_string(),
            title: "网络流行新词".to_string(),
            category: "通用".to_string(),
            description: "搜狗搜索自动生成的流行新词，每周更新。".to_string(),
            source: "搜狗细胞词库".to_string(),
            source_name: "sogou_trending.scel".to_string(),
            detail_url: "https://pinyin.sogou.com/dict/detail/index/4".to_string(),
        },
        OnlineDictionary {
            id: "sogou_computer_all".to_string(),
            title: "计算机词汇大全".to_string(),
            category: "技术".to_string(),
            description: "计算机、软件、网络和信息技术常用词。".to_string(),
            source: "搜狗细胞词库".to_string(),
            source_name: "sogou_computer_all.scel".to_string(),
            detail_url: "https://pinyin.sogou.com/dict/detail/index/15117".to_string(),
        },
        OnlineDictionary {
            id: "sogou_developer".to_string(),
            title: "开发大神专用词库".to_string(),
            category: "技术".to_string(),
            description: "程序开发、配置、资源、工具链等开发者常用词。".to_string(),
            source: "搜狗细胞词库".to_string(),
            source_name: "sogou_developer.scel".to_string(),
            detail_url: "https://pinyin.sogou.com/dict/detail/index/75228".to_string(),
        },
        OnlineDictionary {
            id: "sogou_programming_all".to_string(),
            title: "计算机编程开发词库大全".to_string(),
            category: "技术".to_string(),
            description: "覆盖 Python、C、JavaScript、Web 全栈和常用算法词汇。".to_string(),
            source: "搜狗细胞词库".to_string(),
            source_name: "sogou_programming_all.scel".to_string(),
            detail_url: "https://pinyin.sogou.com/dict/detail/index/133021".to_string(),
        },
        OnlineDictionary {
            id: "sogou_security_3k".to_string(),
            title: "网络安全词库3k".to_string(),
            category: "技术".to_string(),
            description: "网络安全、渗透测试、数据安全、运维等常用词。".to_string(),
            source: "搜狗细胞词库".to_string(),
            source_name: "sogou_security_3k.scel".to_string(),
            detail_url: "https://pinyin.sogou.com/dict/detail/index/169975".to_string(),
        },
        OnlineDictionary {
            id: "sogou_car_models".to_string(),
            title: "汽车车型大全".to_string(),
            category: "生活".to_string(),
            description: "汽车品牌、厂商和车型名称。".to_string(),
            source: "搜狗细胞词库".to_string(),
            source_name: "sogou_car_models.scel".to_string(),
            detail_url: "https://pinyin.sogou.com/dict/detail/index/22422".to_string(),
        },
        OnlineDictionary {
            id: "sogou_high_school_poems".to_string(),
            title: "高中常考古诗词".to_string(),
            category: "文史".to_string(),
            description: "高中阶段常见古诗词句。".to_string(),
            source: "搜狗细胞词库".to_string(),
            source_name: "sogou_high_school_poems.scel".to_string(),
            detail_url: "https://pinyin.sogou.com/dict/detail/index/77212".to_string(),
        },
        OnlineDictionary {
            id: "sogou_middle_school_poems".to_string(),
            title: "中学常见古诗词".to_string(),
            category: "文史".to_string(),
            description: "中学阶段常见古诗词句。".to_string(),
            source: "搜狗细胞词库".to_string(),
            source_name: "sogou_middle_school_poems.scel".to_string(),
            detail_url: "https://pinyin.sogou.com/dict/detail/index/74594".to_string(),
        },
    ]
}

pub(crate) fn html_entity_decode(value: &str) -> String {
    value
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

pub(crate) fn strip_html_tags(value: &str) -> String {
    let mut output = String::new();
    let mut in_tag = false;
    for ch in value.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => output.push(ch),
            _ => {}
        }
    }
    html_entity_decode(output.trim())
}

pub(crate) fn find_between<'a>(contents: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let start_index = contents.find(start)? + start.len();
    let rest = &contents[start_index..];
    let end_index = rest.find(end)?;
    Some(&rest[..end_index])
}

pub(crate) fn extract_detail_title(block: &str) -> Option<(String, String)> {
    let link = find_between(block, "<div class=\"detail_title\"><a href='", "</a>")?;
    let (href, title_part) = link.split_once("'>")?;
    let id = href.rsplit('/').next()?.trim();
    if id.is_empty() {
        return None;
    }
    Some((id.to_string(), strip_html_tags(title_part)))
}

pub(crate) fn extract_show_contents(block: &str) -> Vec<String> {
    let mut contents = Vec::new();
    let mut rest = block;
    while let Some(start) = rest.find("<div class=\"show_content\">") {
        rest = &rest[start + "<div class=\"show_content\">".len()..];
        let Some(end) = rest.find("</div>") else {
            break;
        };
        contents.push(strip_html_tags(&rest[..end]));
        rest = &rest[end + "</div>".len()..];
    }
    contents
}

pub(crate) fn parse_sogou_category_page(category_id: &str, html: &str) -> Vec<OnlineDictionary> {
    let category_title = online_dictionary_categories()
        .into_iter()
        .find(|category| category.id == category_id)
        .map(|category| category.title)
        .unwrap_or_else(|| "搜狗分类".to_string());
    let mut result = Vec::new();

    for block in html.split("<div class=\"dict_detail_block").skip(1) {
        if let Some((id, title)) = extract_detail_title(block) {
            let show_contents = extract_show_contents(block);
            let sample = show_contents.first().cloned().unwrap_or_default();
            let downloads = show_contents.get(1).cloned().unwrap_or_default();
            let updated = show_contents.get(2).cloned().unwrap_or_default();
            let mut description = sample;
            if !downloads.is_empty() || !updated.is_empty() {
                description = format!(
                    "{}{}{}",
                    description,
                    if downloads.is_empty() {
                        "".to_string()
                    } else {
                        format!(" 下载 {downloads}")
                    },
                    if updated.is_empty() {
                        "".to_string()
                    } else {
                        format!(" 更新 {updated}")
                    }
                )
                .trim()
                .to_string();
            }
            result.push(OnlineDictionary {
                id: format!("sogou_{id}"),
                title,
                category: category_title.clone(),
                description,
                source: "搜狗细胞词库".to_string(),
                source_name: format!("sogou_{id}.scel"),
                detail_url: format!("https://pinyin.sogou.com/dict/detail/index/{id}"),
            });
        }
    }

    result
}

pub(crate) fn sogou_detail_id(detail_url: &str) -> Option<&str> {
    detail_url
        .rsplit('/')
        .next()
        .filter(|value| !value.is_empty())
}

pub(crate) fn online_dictionary_by_id(id: &str) -> Option<OnlineDictionary> {
    online_dictionary_catalog()
        .into_iter()
        .find(|entry| entry.id == id)
}

pub(crate) fn dictionary_source_name_from_url(url: &str, fallback: &str) -> String {
    let without_query = url.split(['?', '#']).next().unwrap_or(url);
    let name = without_query
        .rsplit('/')
        .next()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(fallback);
    if name.contains('.') {
        name.to_string()
    } else {
        fallback.to_string()
    }
}

pub(crate) fn validate_dictionary_download_url(url: &str) -> Result<(), RimeError> {
    let lower = url.to_lowercase();
    if lower.starts_with("https://") || lower.starts_with("http://") {
        Ok(())
    } else {
        Err(RimeError::InvalidDictionaryPath(
            "只支持 http:// 或 https:// 在线词库地址".to_string(),
        ))
    }
}

pub(crate) fn download_dictionary_bytes(
    url: &str,
    referer: Option<&str>,
) -> Result<Vec<u8>, RimeError> {
    validate_dictionary_download_url(url)?;
    let mut request = http_agent()
        .get(url)
        .set("User-Agent", "RimeStudio/0.4")
        .set("Accept", "*/*");
    if let Some(referer) = referer {
        request = request.set("Referer", referer);
    }

    let response = request
        .call()
        .map_err(|err| RimeError::NetworkError(format!("下载在线词库失败: {err}")))?;
    let mut reader = response
        .into_reader()
        .take((MAX_DICTIONARY_DOWNLOAD_BYTES + 1) as u64);
    let mut data = Vec::new();
    reader
        .read_to_end(&mut data)
        .map_err(|err| RimeError::NetworkError(format!("读取在线词库失败: {err}")))?;
    if data.is_empty() {
        return Err(RimeError::DownloadError("在线词库下载结果为空".to_string()));
    }
    if data.len() > MAX_DICTIONARY_DOWNLOAD_BYTES {
        return Err(RimeError::DownloadError(
            "在线词库超过 64MB，已取消导入".to_string(),
        ));
    }
    Ok(data)
}

pub(crate) fn download_url_to_file_with_progress<F>(
    url: &str,
    destination: &Path,
    max_bytes: usize,
    empty_message: &str,
    too_large_message: &str,
    mut progress: F,
) -> Result<(), RimeError>
where
    F: FnMut(u64, Option<u64>),
{
    validate_dictionary_download_url(url)?;
    let response = http_agent()
        .get(url)
        .set("User-Agent", "RimeStudio/0.4")
        .set("Accept", "*/*")
        .call()
        .map_err(|err| RimeError::DownloadError(format!("下载失败: {err}")))?;
    let total_bytes = response
        .header("Content-Length")
        .and_then(|value| value.parse::<u64>().ok());
    if total_bytes.is_some_and(|value| value > max_bytes as u64) {
        return Err(RimeError::DownloadError(too_large_message.to_string()));
    }

    let mut reader = response.into_reader();
    let mut file = fs::File::create(destination)
        .map_err(|err| RimeError::FileOperationError(format!("创建下载文件失败: {err}")))?;
    let mut buffer = [0u8; 64 * 1024];
    let mut downloaded = 0u64;
    let mut last_emit = Instant::now();
    progress(downloaded, total_bytes);

    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|err| RimeError::DownloadError(format!("读取下载内容失败: {err}")))?;
        if read == 0 {
            break;
        }
        downloaded += read as u64;
        if downloaded > max_bytes as u64 {
            let _ = fs::remove_file(destination);
            return Err(RimeError::DownloadError(too_large_message.to_string()));
        }
        file.write_all(&buffer[..read])
            .map_err(|err| RimeError::FileOperationError(format!("保存下载文件失败: {err}")))?;
        if last_emit.elapsed().as_millis() >= 200 || total_bytes == Some(downloaded) {
            progress(downloaded, total_bytes);
            last_emit = Instant::now();
        }
    }

    if downloaded == 0 {
        let _ = fs::remove_file(destination);
        return Err(RimeError::DownloadError(empty_message.to_string()));
    }
    file.flush()
        .map_err(|err| RimeError::FileOperationError(format!("保存下载文件失败: {err}")))?;
    progress(downloaded, total_bytes);
    Ok(())
}

pub(crate) fn resolve_sogou_detail_download(
    url: &str,
    source_name: Option<String>,
) -> Result<Option<(String, String, Option<String>)>, RimeError> {
    let marker = "pinyin.sogou.com/dict/detail/index/";
    let Some(marker_index) = url.find(marker) else {
        return Ok(None);
    };
    let id = url[marker_index + marker.len()..]
        .split(['?', '#', '/'])
        .next()
        .unwrap_or_default()
        .trim();
    if id.is_empty() {
        return Err(RimeError::InvalidDictionaryPath(
            "搜狗词库详情页地址缺少词库 ID".to_string(),
        ));
    }

    let response = http_agent()
        .get(url)
        .set("User-Agent", "RimeStudio/0.4")
        .set("Accept", "text/html,*/*")
        .call()
        .map_err(|err| RimeError::NetworkError(format!("读取搜狗词库详情页失败: {err}")))?;
    let html = response
        .into_string()
        .map_err(|err| RimeError::NetworkError(format!("解析搜狗词库详情页失败: {err}")))?;
    let title = find_between(&html, "<div class=\"dict_detail_title\">", "</div>")
        .map(strip_html_tags)
        .or_else(|| {
            find_between(&html, "<title>", "</title>")
                .map(strip_html_tags)
                .map(|title| title.replace("_搜狗输入法词库", ""))
        })
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| format!("搜狗词库 {id}"));
    let source_name = source_name
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| format!("sogou_{id}.scel"));
    Ok(Some((
        sogou_dictionary_url(id, &title),
        source_name,
        Some(url.to_string()),
    )))
}

pub(crate) fn download_dictionary_import_source(
    url: String,
    source_name: Option<String>,
) -> Result<(String, Vec<u8>), RimeError> {
    if let Some((download_url, source_name, referer)) =
        resolve_sogou_detail_download(&url, source_name.clone())?
    {
        let data = download_dictionary_bytes(&download_url, referer.as_deref())?;
        return Ok((source_name, data));
    }

    let source_name = source_name
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| dictionary_source_name_from_url(&url, "online_dictionary.scel"));
    let data = download_dictionary_bytes(&url, None)?;
    Ok((source_name, data))
}

pub(crate) fn download_online_dictionary(entry: &OnlineDictionary) -> Result<Vec<u8>, RimeError> {
    let detail_id = sogou_detail_id(&entry.detail_url)
        .ok_or_else(|| RimeError::InvalidDictionaryPath("在线词库详情地址无效".to_string()))?;
    let url = sogou_dictionary_url(detail_id, &entry.title);
    download_dictionary_bytes(&url, Some(&entry.detail_url))
}

pub(crate) fn list_online_dictionaries_sync() -> Result<Vec<OnlineDictionary>, RimeError> {
    Ok(online_dictionary_catalog())
}

pub(crate) fn list_online_dictionary_categories_sync(
) -> Result<Vec<OnlineDictionaryCategory>, RimeError> {
    Ok(online_dictionary_categories())
}

pub(crate) fn list_online_dictionaries_by_category_sync(
    category_id: String,
) -> Result<Vec<OnlineDictionary>, RimeError> {
    let category = online_dictionary_categories()
        .into_iter()
        .find(|category| category.id == category_id)
        .ok_or_else(|| RimeError::NetworkError("未找到在线词库分类".to_string()))?;
    let url = format!(
        "https://pinyin.sogou.com/dict/cate/index/{}/download",
        category.id
    );
    let response = http_agent()
        .get(&url)
        .set("User-Agent", "RimeStudio/0.4")
        .set("Accept", "text/html,*/*")
        .call()
        .map_err(|err| RimeError::NetworkError(format!("读取在线词库分类失败: {err}")))?;
    let html = response
        .into_string()
        .map_err(|err| RimeError::NetworkError(format!("解析在线词库分类失败: {err}")))?;
    let dictionaries = parse_sogou_category_page(&category.id, &html);
    if dictionaries.is_empty() {
        Err(RimeError::NetworkError(
            "这个分类没有解析到可导入词库".to_string(),
        ))
    } else {
        Ok(dictionaries)
    }
}

pub(crate) fn preview_online_dictionary_import_sync(
    id: String,
) -> Result<DictionaryImportPreview, RimeError> {
    let entry = online_dictionary_by_id(&id)
        .ok_or_else(|| RimeError::NetworkError("未找到在线词库".to_string()))?;
    let data = download_online_dictionary(&entry)?;
    preview_dictionary_import_sync(entry.source_name, data)
}

pub(crate) fn import_online_dictionary_sync(
    id: String,
) -> Result<DictionaryImportResult, RimeError> {
    let entry = online_dictionary_by_id(&id)
        .ok_or_else(|| RimeError::NetworkError("未找到在线词库".to_string()))?;
    let data = download_online_dictionary(&entry)?;
    import_dictionary_sync(entry.source_name, data)
}

pub(crate) fn preview_dictionary_url_import_sync(
    url: String,
    source_name: Option<String>,
) -> Result<DictionaryImportPreview, RimeError> {
    let (source_name, data) = download_dictionary_import_source(url, source_name)?;
    preview_dictionary_import_sync(source_name, data)
}

pub(crate) fn import_dictionary_url_sync(
    url: String,
    source_name: Option<String>,
) -> Result<DictionaryImportResult, RimeError> {
    let (source_name, data) = download_dictionary_import_source(url, source_name)?;
    import_dictionary_sync(source_name, data)
}
