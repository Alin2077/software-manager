use serde::Serialize;
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Clone, Serialize)]
pub struct FileEntry {
    pub name: String,
    pub path: String,
    pub is_dir: bool,
    pub size: u64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<FileEntry>,
}

/// 扫描目录，返回文件树。限制深度和条目数以防止卡死。
pub fn scan_directory(path: &str) -> Option<FileEntry> {
    let root = Path::new(path);
    if !root.exists() || !root.is_dir() {
        return None;
    }

    let max_depth = 8;
    let max_entries = 2000;
    let root_name = root
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string());

    let mut root_entry = FileEntry {
        name: root_name,
        path: path.to_string(),
        is_dir: true,
        size: 0,
        children: Vec::new(),
    };

    let mut entry_count = 0;

    for entry in WalkDir::new(path)
        .max_depth(max_depth)
        .follow_links(false)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry_count >= max_entries {
            break;
        }
        entry_count += 1;

        // 跳过根目录自身
        if entry.path() == root {
            continue;
        }

        let file_name = entry
            .file_name()
            .to_string_lossy()
            .to_string();

        let full_path = entry.path().to_string_lossy().to_string();
        let is_dir = entry.file_type().is_dir();
        let size = if is_dir {
            0
        } else {
            entry.metadata().map(|m| m.len()).unwrap_or(0)
        };

        let entry_node = FileEntry {
            name: file_name,
            path: full_path,
            is_dir,
            size,
            children: Vec::new(),
        };

        let parent_path = entry
            .path()
            .parent()
            .map(|p| p.to_string_lossy())
            .unwrap_or_default();

        insert_into_tree(&mut root_entry, &parent_path, &entry_node);
    }

    Some(root_entry)
}

/// 递归查找父节点并插入（DFS 先序保证父目录已存在）。
fn insert_into_tree(node: &mut FileEntry, target_parent: &str, entry: &FileEntry) -> bool {
    if node.path.eq_ignore_ascii_case(target_parent) && node.is_dir {
        node.children.push(entry.clone());
        return true;
    }

    if node.is_dir {
        let node_path_lower = node.path.to_lowercase();
        let target_lower = target_parent.to_lowercase();
        if target_lower.starts_with(&node_path_lower) {
            let len = node.children.len();
            for i in 0..len {
                if insert_into_tree(&mut node.children[i], target_parent, entry) {
                    return true;
                }
            }
        }
    }

    false
}
