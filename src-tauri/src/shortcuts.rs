use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize)]
pub struct ShortcutInfo {
    pub name: String,
    pub path: String,
    pub target: String,
}

/// 查找与指定路径相关的所有 .lnk 快捷方式
pub fn find_shortcuts_for_path(target_path: &str) -> Vec<ShortcutInfo> {
    let mut results = Vec::new();
    let target_lower = target_path.to_lowercase();

    // 扫描开始菜单和桌面
    let scan_dirs = vec![
        // 当前用户开始菜单
        dirs_start_menu(),
        // 公共开始菜单
        dirs_common_start_menu(),
        // 当前用户桌面
        dirs_desktop(),
        // 公共桌面
        dirs_common_desktop(),
    ];

    for dir in scan_dirs {
        if let Some(d) = dir {
            scan_dir_for_lnk(&d, &target_lower, &mut results, 2);
        }
    }

    results
}

fn scan_dir_for_lnk(dir: &Path, target_lower: &str, results: &mut Vec<ShortcutInfo>, max_depth: usize) {
    if max_depth == 0 {
        return;
    }
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path
                .file_stem()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();

            if path.is_dir() {
                scan_dir_for_lnk(&path, target_lower, results, max_depth - 1);
            } else if path
                .extension()
                .map(|e| e.to_ascii_lowercase() == "lnk")
                .unwrap_or(false)
            {
                if let Some(lnk_target) = parse_lnk_target(&path) {
                    if lnk_target.to_lowercase().contains(target_lower)
                        || target_lower.contains(&lnk_target.to_lowercase())
                    {
                        results.push(ShortcutInfo {
                            name,
                            path: path.to_string_lossy().to_string(),
                            target: lnk_target,
                        });
                    }
                }
            }
        }
    }
}

/// 解析 .lnk 文件，提取目标路径
fn parse_lnk_target(path: &Path) -> Option<String> {
    let data = fs::read(path).ok()?;
    if data.len() < 76 {
        return None;
    }

    // 验证 ShellLink 头
    let header_size = u32_from_le(&data, 0);
    if header_size != 0x4C {
        return None;
    }

    // CLSID: 00021401-0000-0000-C000-000000000046
    let clsid: [u8; 16] = [
        0x01, 0x14, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00,
        0xC0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x46,
    ];
    if &data[4..20] != clsid {
        return None;
    }

    let link_flags = u32_from_le(&data, 20);

    let mut offset: usize = 76; // 跳过固定头

    // --- LinkTargetIDList ---
    if link_flags & 0x01 != 0 {
        if offset + 2 > data.len() {
            return None;
        }
        let id_list_size = u16_from_le(&data, offset) as usize;
        offset += id_list_size;
    }

    // --- LinkInfo ---
    if link_flags & 0x02 != 0 {
        if offset + 28 > data.len() {
            return None;
        }
        let link_info_size = u32_from_le(&data, offset) as usize;
        let link_info_end = offset + link_info_size;
        if link_info_end > data.len() {
            return None;
        }

        let link_info_flags = u32_from_le(&data, offset + 8);
        let local_base_path_offset = u32_from_le(&data, offset + 16) as usize;
        let common_path_suffix_offset = u32_from_le(&data, offset + 24) as usize;

        let is_unicode = link_flags & 0x80 != 0
            || (link_info_flags & 0x01 != 0);

        if local_base_path_offset > 0 {
            let str_offset = offset + local_base_path_offset;
            let target = if is_unicode {
                read_unicode_string(&data, str_offset)
            } else {
                read_ansi_string(&data, str_offset)
            };

            if let Some(t) = target {
                if !t.is_empty() {
                    return Some(t);
                }
            }
        }

        // 如果 LocalBasePath 没有，尝试 CommonPathSuffix
        if common_path_suffix_offset > 0 {
            let str_offset = offset + common_path_suffix_offset;
            let target = if is_unicode {
                read_unicode_string(&data, str_offset)
            } else {
                read_ansi_string(&data, str_offset)
            };
            if let Some(t) = target {
                if !t.is_empty() {
                    return Some(t);
                }
            }
        }

        // link_info_end marks the end of LinkInfo; not needed further
    }

    None
}

fn u32_from_le(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]])
}

fn u16_from_le(data: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([data[offset], data[offset + 1]])
}

fn read_unicode_string(data: &[u8], offset: usize) -> Option<String> {
    let mut end = offset;
    while end + 1 < data.len() {
        if data[end] == 0 && data[end + 1] == 0 {
            break;
        }
        end += 2;
    }
    if end == offset {
        return None;
    }
    let utf16: Vec<u16> = (offset..end)
        .step_by(2)
        .map(|i| u16::from_le_bytes([data[i], data[i + 1]]))
        .collect();
    String::from_utf16(&utf16).ok()
}

fn read_ansi_string(data: &[u8], offset: usize) -> Option<String> {
    let mut end = offset;
    while end < data.len() && data[end] != 0 {
        end += 1;
    }
    if end == offset {
        return None;
    }
    Some(String::from_utf8_lossy(&data[offset..end]).to_string())
}

// --- 路径辅助函数 ---

fn dirs_start_menu() -> Option<std::path::PathBuf> {
    std::env::var("APPDATA")
        .ok()
        .map(|p| Path::new(&p).join("Microsoft").join("Windows").join("Start Menu"))
}

fn dirs_common_start_menu() -> Option<std::path::PathBuf> {
    std::env::var("ProgramData")
        .ok()
        .map(|p| Path::new(&p).join("Microsoft").join("Windows").join("Start Menu"))
}

fn dirs_desktop() -> Option<std::path::PathBuf> {
    std::env::var("USERPROFILE")
        .ok()
        .map(|p| Path::new(&p).join("Desktop"))
}

fn dirs_common_desktop() -> Option<std::path::PathBuf> {
    std::env::var("PUBLIC")
        .ok()
        .map(|p| Path::new(&p).join("Desktop"))
}
