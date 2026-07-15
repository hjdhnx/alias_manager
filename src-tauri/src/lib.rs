mod alias;
mod commands;
mod path_env;
mod store;

use std::collections::HashSet;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::Manager;

/// 全局路径：别名数据文件 + shim 所在 bin 目录
pub struct Paths {
    pub data_file: PathBuf,
    pub bin_dir: PathBuf,
}

/// 清理 bin 目录中的孤儿 shim（不在当前启用别名集合里的 .cmd 文件）
fn cleanup_orphan_shims(bin_dir: &std::path::Path, store: &store::Store) {
    let known: HashSet<String> = store
        .aliases
        .iter()
        .filter(|a| a.enabled)
        .map(|a| alias::shim_filename(&a.name))
        .collect();
    if let Ok(entries) = std::fs::read_dir(bin_dir) {
        for entry in entries.flatten() {
            if let Some(name) = entry.file_name().to_str() {
                if name.ends_with(".cmd") && !known.contains(name) {
                    let _ = std::fs::remove_file(entry.path());
                }
            }
        }
    }
}

pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let dir = app.path().app_local_data_dir()?;
            std::fs::create_dir_all(&dir).ok();
            let bin_dir = dir.join("bin");
            std::fs::create_dir_all(&bin_dir).ok();
            let data_file = dir.join("aliases.json");

            let store = store::Store::load(&data_file);
            // 启动时按数据状态同步 shim，保证磁盘与数据一致
            for a in &store.aliases {
                if let Err(e) = alias::sync_shim(&bin_dir, a) {
                    eprintln!("启动同步 shim 失败 ({}): {e}", a.name);
                }
            }
            cleanup_orphan_shims(&bin_dir, &store);

            app.manage(Mutex::new(store));
            app.manage(Paths { data_file, bin_dir });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::list_aliases,
            commands::add_alias,
            commands::update_alias,
            commands::delete_alias,
            commands::toggle_alias,
            commands::get_status,
            commands::ensure_path,
            commands::open_bin_dir,
            commands::test_alias,
            commands::export_aliases,
            commands::import_aliases,
            commands::set_enabled,
            commands::delete_aliases,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
