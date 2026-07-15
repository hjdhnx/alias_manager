use crate::alias::Alias;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// 别名数据根结构，序列化为 aliases.json
#[derive(Serialize, Deserialize, Default, Clone)]
pub struct Store {
    #[serde(default)]
    pub aliases: Vec<Alias>,
}

impl Store {
    /// 从磁盘加载；文件缺失或解析失败时返回空 Store（不报错，保证启动健壮）
    pub fn load(path: &Path) -> Self {
        match fs::read_to_string(path) {
            Ok(content) if !content.trim().is_empty() => {
                serde_json::from_str(&content).unwrap_or_default()
            }
            _ => Store::default(),
        }
    }

    /// 持久化到磁盘（自动创建父目录，pretty 便于人工查看）
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let parent = path.parent().ok_or("无法确定配置文件父目录")?;
        fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {e}"))?;
        let json =
            serde_json::to_string_pretty(self).map_err(|e| format!("序列化失败: {e}"))?;
        fs::write(path, json).map_err(|e| format!("写入配置失败: {e}"))
    }
}
