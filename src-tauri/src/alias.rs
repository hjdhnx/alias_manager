use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// 一条别名定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alias {
    pub id: String,
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub created_at: String,
    pub updated_at: String,
}

fn default_true() -> bool {
    true
}

/// Windows 保留设备名，禁止作为别名
const RESERVED: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// 校验别名名称：仅字母/数字/下划线/连字符，非空，非保留名
pub fn validate_name(name: &str) -> Result<(), String> {
    let n = name.trim();
    if n.is_empty() {
        return Err("别名名称不能为空".into());
    }
    if !n
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err("名称仅允许字母、数字、下划线和连字符".into());
    }
    let upper = n.to_uppercase();
    if RESERVED.contains(&upper.as_str()) {
        return Err(format!("名称 {n} 是 Windows 保留名，请更换"));
    }
    Ok(())
}

/// 校验命令非空
pub fn validate_command(cmd: &str) -> Result<(), String> {
    if cmd.trim().is_empty() {
        return Err("命令不能为空".into());
    }
    Ok(())
}

/// shim 文件名：<name>.cmd
pub fn shim_filename(name: &str) -> String {
    format!("{name}.cmd")
}

/// 生成 shim 文件内容。
/// `call` 确保目标若是批处理也能正确返回并转发退出码；`%*` 透传用户额外参数。
pub fn shim_content(command: &str) -> String {
    format!("@echo off\r\ncall {command} %*\r\n")
}

/// 生成启用别名的 shim 文件
pub fn write_shim(bin_dir: &Path, alias: &Alias) -> Result<(), String> {
    let file = bin_dir.join(shim_filename(&alias.name));
    fs::write(&file, shim_content(&alias.command))
        .map_err(|e| format!("生成 shim 失败 ({}): {e}", file.display()))
}

/// 删除 shim（不存在视为成功）
pub fn remove_shim(bin_dir: &Path, name: &str) -> Result<(), String> {
    let file = bin_dir.join(shim_filename(name));
    if file.exists() {
        fs::remove_file(&file).map_err(|e| format!("删除 shim 失败: {e}"))?;
    }
    Ok(())
}

/// 按 enabled 状态同步 shim：启用→生成，禁用→删除
pub fn sync_shim(bin_dir: &Path, alias: &Alias) -> Result<(), String> {
    if alias.enabled {
        write_shim(bin_dir, alias)
    } else {
        remove_shim(bin_dir, &alias.name)
    }
}

/// 导出/导入用的别名条目（不含 id 与时间戳，导入时重建）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportItem {
    pub name: String,
    pub command: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl From<&Alias> for ExportItem {
    fn from(a: &Alias) -> Self {
        ExportItem {
            name: a.name.clone(),
            command: a.command.clone(),
            description: a.description.clone(),
            enabled: a.enabled,
        }
    }
}

/// 分享载荷：带 app 标识与版本，便于校验与未来兼容
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportPayload {
    pub app: String,
    pub version: u32,
    pub aliases: Vec<ExportItem>,
}

pub const EXPORT_APP: &str = "alias-manager";
pub const EXPORT_VERSION: u32 = 1;
