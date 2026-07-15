use crate::alias::{self, Alias, ExportItem, ExportPayload, EXPORT_APP, EXPORT_VERSION};
use crate::path_env;
use crate::store::Store;
use crate::Paths;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use chrono::Utc;
use serde::Serialize;
use std::sync::Mutex;
use tauri::State;
use uuid::Uuid;

/// 运行期状态：PATH 配置情况与统计
#[derive(Serialize)]
pub struct StatusInfo {
    pub bin_dir: String,
    pub path_configured: bool,
    pub total: usize,
    pub enabled: usize,
}

/// test_alias 返回：shim 是否存在、bin 是否在 PATH、是否可用
#[derive(Serialize)]
pub struct TestResult {
    pub shim_exists: bool,
    pub bin_in_path: bool,
    pub available: bool,
}

/// import_aliases 返回：成功数、跳过数、跳过的名称
#[derive(Serialize)]
pub struct ImportResult {
    pub imported: usize,
    pub skipped: usize,
    pub skipped_names: Vec<String>,
}

fn now_rfc3339() -> String {
    Utc::now().to_rfc3339()
}

/// 规整描述：去空白，空串转 None
fn norm_desc(raw: &str) -> Option<String> {
    let d = raw.trim();
    if d.is_empty() {
        None
    } else {
        Some(d.to_string())
    }
}

#[tauri::command]
pub fn list_aliases(store: State<'_, Mutex<Store>>) -> Vec<Alias> {
    store.lock().unwrap().aliases.clone()
}

#[tauri::command]
pub fn add_alias(
    name: String,
    command: String,
    description: String,
    store: State<'_, Mutex<Store>>,
    paths: State<'_, Paths>,
) -> Result<Alias, String> {
    let name = name.trim().to_string();
    let command = command.trim().to_string();
    let desc = norm_desc(&description);

    alias::validate_name(&name)?;
    alias::validate_command(&command)?;

    let mut s = store.lock().unwrap();
    if s.aliases.iter().any(|a| a.name.eq_ignore_ascii_case(&name)) {
        return Err(format!("别名 {name} 已存在"));
    }
    let now = now_rfc3339();
    let alias = Alias {
        id: Uuid::new_v4().to_string(),
        name,
        command,
        description: desc,
        enabled: true,
        created_at: now.clone(),
        updated_at: now,
    };
    alias::write_shim(&paths.bin_dir, &alias)?;
    s.aliases.push(alias.clone());
    s.save(&paths.data_file)?;
    Ok(alias)
}

#[tauri::command]
pub fn update_alias(
    id: String,
    name: String,
    command: String,
    description: String,
    store: State<'_, Mutex<Store>>,
    paths: State<'_, Paths>,
) -> Result<Alias, String> {
    let name = name.trim().to_string();
    let command = command.trim().to_string();
    let desc = norm_desc(&description);

    alias::validate_name(&name)?;
    alias::validate_command(&command)?;

    let mut s = store.lock().unwrap();
    let idx = s
        .aliases
        .iter()
        .position(|a| a.id == id)
        .ok_or("别名不存在")?;
    if s
        .aliases
        .iter()
        .any(|a| a.id != id && a.name.eq_ignore_ascii_case(&name))
    {
        return Err(format!("别名 {name} 已存在"));
    }

    let old_name = s.aliases[idx].name.clone();
    let name_changed = old_name != name;

    let target = &mut s.aliases[idx];
    target.name = name.clone();
    target.command = command.clone();
    target.description = desc;
    target.updated_at = now_rfc3339();
    let updated = target.clone();

    // 改名时先删旧 shim，再按当前 enabled 状态重建/删除
    if name_changed {
        alias::remove_shim(&paths.bin_dir, &old_name)?;
    }
    alias::sync_shim(&paths.bin_dir, &updated)?;

    s.save(&paths.data_file)?;
    Ok(updated)
}

#[tauri::command]
pub fn delete_alias(
    id: String,
    store: State<'_, Mutex<Store>>,
    paths: State<'_, Paths>,
) -> Result<(), String> {
    let mut s = store.lock().unwrap();
    let idx = s
        .aliases
        .iter()
        .position(|a| a.id == id)
        .ok_or("别名不存在")?;
    let removed = s.aliases.remove(idx);
    alias::remove_shim(&paths.bin_dir, &removed.name)?;
    s.save(&paths.data_file)?;
    Ok(())
}

#[tauri::command]
pub fn toggle_alias(
    id: String,
    enabled: bool,
    store: State<'_, Mutex<Store>>,
    paths: State<'_, Paths>,
) -> Result<Alias, String> {
    let mut s = store.lock().unwrap();
    let idx = s
        .aliases
        .iter()
        .position(|a| a.id == id)
        .ok_or("别名不存在")?;
    let target = &mut s.aliases[idx];
    target.enabled = enabled;
    target.updated_at = now_rfc3339();
    let updated = target.clone();
    alias::sync_shim(&paths.bin_dir, &updated)?;
    s.save(&paths.data_file)?;
    Ok(updated)
}

#[tauri::command]
pub fn get_status(store: State<'_, Mutex<Store>>, paths: State<'_, Paths>) -> StatusInfo {
    let s = store.lock().unwrap();
    StatusInfo {
        bin_dir: paths.bin_dir.to_string_lossy().to_string(),
        path_configured: path_env::is_in_path(&paths.bin_dir).unwrap_or(false),
        total: s.aliases.len(),
        enabled: s.aliases.iter().filter(|a| a.enabled).count(),
    }
}

#[tauri::command]
pub fn ensure_path(paths: State<'_, Paths>) -> Result<bool, String> {
    path_env::ensure_in_path(&paths.bin_dir)
}

#[tauri::command]
pub fn open_bin_dir(paths: State<'_, Paths>) -> Result<(), String> {
    let p = paths.bin_dir.to_string_lossy().to_string();
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("explorer.exe")
            .arg(&p)
            .spawn()
            .map_err(|e| format!("打开 bin 目录失败: {e}"))?;
        Ok(())
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = p;
        Err("仅支持 Windows".into())
    }
}

#[tauri::command]
pub fn test_alias(name: String, paths: State<'_, Paths>) -> Result<TestResult, String> {
    let shim = paths.bin_dir.join(alias::shim_filename(name.trim()));
    let shim_exists = shim.exists();
    let bin_in_path = path_env::is_in_path(&paths.bin_dir)?;
    Ok(TestResult {
        shim_exists,
        bin_in_path,
        available: shim_exists && bin_in_path,
    })
}

/// 把选中别名序列化为 JSON 再 base64 编码，作为可分享文本
#[tauri::command]
pub fn export_aliases(ids: Vec<String>, store: State<'_, Mutex<Store>>) -> Result<String, String> {
    let s = store.lock().unwrap();
    let items: Vec<ExportItem> = s
        .aliases
        .iter()
        .filter(|a| ids.iter().any(|id| id == &a.id))
        .map(ExportItem::from)
        .collect();
    if items.is_empty() {
        return Err("没有选中的别名".into());
    }
    let payload = ExportPayload {
        app: EXPORT_APP.to_string(),
        version: EXPORT_VERSION,
        aliases: items,
    };
    let json = serde_json::to_string(&payload).map_err(|e| format!("序列化失败: {e}"))?;
    Ok(STANDARD.encode(json))
}

/// 解析分享文本（base64），逐条导入；重名或无效项跳过并计数
#[tauri::command]
pub fn import_aliases(
    data: String,
    store: State<'_, Mutex<Store>>,
    paths: State<'_, Paths>,
) -> Result<ImportResult, String> {
    let data = data.trim();
    let bytes = STANDARD
        .decode(data)
        .map_err(|e| format!("Base64 解码失败：{e}"))?;
    let json = String::from_utf8(bytes).map_err(|e| format!("UTF-8 解码失败：{e}"))?;
    let payload: ExportPayload =
        serde_json::from_str(&json).map_err(|e| format!("解析分享数据失败：{e}"))?;

    let mut s = store.lock().unwrap();
    let mut imported = 0usize;
    let mut skipped = 0usize;
    let mut skipped_names: Vec<String> = Vec::new();

    for item in payload.aliases {
        if alias::validate_name(&item.name).is_err()
            || alias::validate_command(&item.command).is_err()
        {
            skipped += 1;
            skipped_names.push(item.name);
            continue;
        }
        // 重名（大小写不敏感）跳过
        if s.aliases.iter().any(|a| a.name.eq_ignore_ascii_case(&item.name)) {
            skipped += 1;
            skipped_names.push(item.name.clone());
            continue;
        }
        let now = now_rfc3339();
        let alias = Alias {
            id: Uuid::new_v4().to_string(),
            name: item.name,
            command: item.command,
            description: item.description,
            enabled: item.enabled,
            created_at: now.clone(),
            updated_at: now,
        };
        if alias.enabled {
            if let Err(e) = alias::write_shim(&paths.bin_dir, &alias) {
                eprintln!("导入时生成 shim 失败 ({}): {e}", alias.name);
                skipped += 1;
                skipped_names.push(alias.name);
                continue;
            }
        }
        s.aliases.push(alias);
        imported += 1;
    }

    s.save(&paths.data_file)?;
    Ok(ImportResult {
        imported,
        skipped,
        skipped_names,
    })
}

/// 批量启用/禁用：一次落盘，逐条同步 shim，返回受影响数量
#[tauri::command]
pub fn set_enabled(
    ids: Vec<String>,
    enabled: bool,
    store: State<'_, Mutex<Store>>,
    paths: State<'_, Paths>,
) -> Result<usize, String> {
    if ids.is_empty() {
        return Ok(0);
    }
    let mut s = store.lock().unwrap();
    let now = now_rfc3339();
    let mut count = 0usize;
    for a in s.aliases.iter_mut() {
        if ids.contains(&a.id) {
            a.enabled = enabled;
            a.updated_at = now.clone();
            count += 1;
        }
    }
    // 收集受影响项后同步 shim（避开与 iter_mut 的借用冲突）
    let touched: Vec<Alias> = s
        .aliases
        .iter()
        .filter(|a| ids.contains(&a.id))
        .cloned()
        .collect();
    for a in &touched {
        if let Err(e) = alias::sync_shim(&paths.bin_dir, a) {
            eprintln!("同步 shim 失败 ({}): {e}", a.name);
        }
    }
    s.save(&paths.data_file)?;
    Ok(count)
}

/// 批量删除：一次落盘，逐条删 shim，返回删除数量
#[tauri::command]
pub fn delete_aliases(
    ids: Vec<String>,
    store: State<'_, Mutex<Store>>,
    paths: State<'_, Paths>,
) -> Result<usize, String> {
    if ids.is_empty() {
        return Ok(0);
    }
    let mut s = store.lock().unwrap();
    let mut removed: Vec<String> = Vec::new();
    s.aliases.retain(|a| {
        if ids.contains(&a.id) {
            removed.push(a.name.clone());
            false
        } else {
            true
        }
    });
    for name in &removed {
        if let Err(e) = alias::remove_shim(&paths.bin_dir, name) {
            eprintln!("删除 shim 失败 ({name}): {e}");
        }
    }
    s.save(&paths.data_file)?;
    Ok(removed.len())
}
