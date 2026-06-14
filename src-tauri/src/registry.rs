use serde::Serialize;
use winreg::enums::*;
use winreg::HKEY;
use winreg::RegKey;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct Software {
    pub name: String,
    pub version: String,
    pub publisher: String,
    pub install_location: String,
    pub uninstall_string: String,
}

pub fn get_installed_software() -> Vec<Software> {
    let mut map: HashMap<String, Software> = HashMap::new();

    // 读取 64-bit 注册表
    read_uninstall_key(HKEY_LOCAL_MACHINE, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall", &mut map);
    // 读取 32-bit on 64-bit 注册表
    read_uninstall_key(HKEY_LOCAL_MACHINE, r"SOFTWARE\WOW6432Node\Microsoft\Windows\CurrentVersion\Uninstall", &mut map);
    // 当前用户
    read_uninstall_key(HKEY_CURRENT_USER, r"SOFTWARE\Microsoft\Windows\CurrentVersion\Uninstall", &mut map);

    let mut list: Vec<Software> = map.into_values().collect();
    list.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
    list
}

fn read_uninstall_key(hive: HKEY, path: &str, map: &mut HashMap<String, Software>) {
    let hklm = match RegKey::predef(hive).open_subkey_with_flags(path, KEY_READ) {
        Ok(k) => k,
        Err(_) => return,
    };

    for key_name in hklm.enum_keys().filter_map(|k| k.ok()) {
        if let Ok(subkey) = hklm.open_subkey_with_flags(&key_name, KEY_READ) {
            let name: String = subkey.get_value("DisplayName").unwrap_or_default();
            // 跳过空名称和系统组件
            if name.is_empty() {
                continue;
            }

            let version: String = subkey
                .get_value("DisplayVersion")
                .unwrap_or_default();
            let publisher: String = subkey.get_value("Publisher").unwrap_or_default();
            let install_location: String = subkey
                .get_value("InstallLocation")
                .unwrap_or_default();
            let uninstall_string: String = subkey
                .get_value("UninstallString")
                .or_else(|_| subkey.get_value("QuietUninstallString"))
                .unwrap_or_default();

            // 用 name 去重，优先保留有更多信息的版本
            map.entry(name.clone())
                .and_modify(|existing| {
                    if existing.install_location.is_empty() && !install_location.is_empty() {
                        existing.install_location = install_location.clone();
                    }
                    if existing.version.is_empty() && !version.is_empty() {
                        existing.version = version.clone();
                    }
                    if existing.publisher.is_empty() && !publisher.is_empty() {
                        existing.publisher = publisher.clone();
                    }
                    if existing.uninstall_string.is_empty() && !uninstall_string.is_empty() {
                        existing.uninstall_string = uninstall_string.clone();
                    }
                })
                .or_insert(Software {
                    name,
                    version,
                    publisher,
                    install_location,
                    uninstall_string,
                });
        }
    }
}
