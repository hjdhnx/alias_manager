use std::path::Path;

#[cfg(windows)]
mod win {
    use std::path::Path;
    use winreg::enums::*;
    use winreg::{RegKey, RegValue};

    /// 字符串 → UTF-16LE 字节（含 null 终止），用于注册表字符串值
    fn encode_utf16le_z(s: &str) -> Vec<u8> {
        let mut out = Vec::with_capacity((s.len() + 1) * 2);
        for u in s.encode_utf16() {
            out.push((u & 0xff) as u8);
            out.push((u >> 8) as u8);
        }
        out.push(0);
        out.push(0);
        out
    }

    /// UTF-16LE 字节 → 字符串（遇 null 截断，不展开 %VAR%）
    fn utf16le_to_string(bytes: &[u8]) -> String {
        let mut u16s: Vec<u16> = Vec::with_capacity(bytes.len() / 2);
        let mut i = 0;
        while i + 1 < bytes.len() {
            let u = (bytes[i] as u16) | ((bytes[i + 1] as u16) << 8);
            if u == 0 {
                break;
            }
            u16s.push(u);
            i += 2;
        }
        String::from_utf16_lossy(&u16s)
    }

    /// 读取 HKCU\Environment\Path 的原始 RegValue（不展开环境变量）
    fn read_raw_path() -> Result<Option<RegValue>, String> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env = hkcu
            .open_subkey_with_flags("Environment", KEY_READ)
            .map_err(|e| format!("打开 HKCU\\Environment 失败: {e}"))?;
        match env.get_raw_value("Path") {
            Ok(v) => Ok(Some(v)),
            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(format!("读取 Path 失败: {e}")),
        }
    }

    fn read_user_path() -> Result<String, String> {
        Ok(read_raw_path()?
            .map(|v| utf16le_to_string(&v.bytes))
            .unwrap_or_default())
    }

    fn write_user_path(new_path: &str) -> Result<(), String> {
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let env = hkcu
            .open_subkey_with_flags("Environment", KEY_WRITE)
            .map_err(|e| format!("打开 HKCU\\Environment(写) 失败: {e}"))?;
        // 保持原有值类型（通常 REG_EXPAND_SZ），缺失时默认 REG_EXPAND_SZ
        let vtype = read_raw_path()?.map(|v| v.vtype).unwrap_or(RegType::REG_EXPAND_SZ);
        let val = RegValue {
            bytes: encode_utf16le_z(new_path),
            vtype,
        };
        env.set_raw_value("Path", &val)
            .map_err(|e| format!("写入 Path 失败: {e}"))
    }

    /// 广播 WM_SETTINGCHANGE("Environment")，让新启动的进程感知环境变量变化。
    /// 已打开的终端不受影响，仍需重开。
    fn broadcast_env_change() {
        use windows_sys::Win32::Foundation::LPARAM;
        use windows_sys::Win32::UI::WindowsAndMessaging::{
            HWND_BROADCAST, SMTO_ABORTIFHUNG, SendMessageTimeoutW, WM_SETTINGCHANGE,
        };
        let env: Vec<u16> = "Environment\0".encode_utf16().collect();
        unsafe {
            SendMessageTimeoutW(
                HWND_BROADCAST,
                WM_SETTINGCHANGE,
                0,
                env.as_ptr() as LPARAM,
                SMTO_ABORTIFHUNG,
                5000,
                std::ptr::null_mut(),
            );
        }
    }

    pub fn ensure_in_path(bin_dir: &Path) -> Result<bool, String> {
        let bin_str = bin_dir.to_string_lossy().to_string();
        let current = read_user_path()?;
        if super::path_contains(&current, &bin_str) {
            return Ok(false);
        }
        let new_path = if current.trim().is_empty() {
            bin_str
        } else if current.ends_with(';') {
            format!("{current}{bin_str}")
        } else {
            format!("{current};{bin_str}")
        };
        write_user_path(&new_path)?;
        broadcast_env_change();
        Ok(true)
    }

    pub fn is_in_path(bin_dir: &Path) -> Result<bool, String> {
        let current = read_user_path()?;
        Ok(super::path_contains(&current, &bin_dir.to_string_lossy()))
    }
}

#[cfg(not(windows))]
mod win {
    use std::path::Path;
    pub fn ensure_in_path(_: &Path) -> Result<bool, String> {
        Err("当前系统不支持自动配置 PATH".into())
    }
    pub fn is_in_path(_: &Path) -> Result<bool, String> {
        Ok(false)
    }
}

/// 规范化路径：去首尾空白、去尾部路径分隔符、转小写，用于 PATH 比较
fn normalize(p: &str) -> String {
    p.trim().trim_end_matches(['\\', '/']).to_lowercase()
}

/// 判断 PATH 字符串中是否已包含指定目录
fn path_contains(path_value: &str, dir: &str) -> bool {
    let dir_norm = normalize(dir);
    if dir_norm.is_empty() {
        return false;
    }
    path_value
        .split(';')
        .any(|p| normalize(p) == dir_norm)
}

pub fn ensure_in_path(bin_dir: &Path) -> Result<bool, String> {
    win::ensure_in_path(bin_dir)
}

pub fn is_in_path(bin_dir: &Path) -> Result<bool, String> {
    win::is_in_path(bin_dir)
}
