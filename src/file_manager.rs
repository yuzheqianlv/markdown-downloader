use std::fs;
use std::path::Path;
use url::Url;
use chrono::Utc;
use crate::error::Result;

pub struct FileManager {
    output_dir: String,
}

impl FileManager {
    pub fn new(output_dir: String) -> Result<Self> {
        fs::create_dir_all(&output_dir)?;
        Ok(Self { output_dir })
    }

    pub fn save_markdown(&self, url: &str, content: &str) -> Result<String> {
        let filename = self.generate_filename(url)?;
        let filepath = Path::new(&self.output_dir).join(&filename);
        
        fs::write(&filepath, content)?;
        Ok(filename)
    }

    fn generate_filename(&self, url: &str) -> Result<String> {
        let parsed_url = Url::parse(url)?;
        let domain = parsed_url.host_str().unwrap_or("unknown");
        let path = parsed_url.path();

        // 获取当前日期
        let now = Utc::now();
        let date = now.format("%Y%m%d").to_string();

        // 清理路径，替换特殊字符
        let temp_path = path
            .replace('/', "_")
            .replace('\\', "_")
            .replace('?', "_")
            .replace('&', "_")
            .replace('=', "_")
            .replace('#', "_");

        let clean_path = temp_path.trim_matches('_');

        let clean_path = if clean_path.is_empty() {
            "index".to_string()
        } else {
            clean_path.to_string()
        };

        // 限制文件名长度
        let clean_path = if clean_path.len() > 100 {
            clean_path[..100].to_string()
        } else {
            clean_path
        };

        Ok(format!(
            "{}_{}{}.md",
            domain,
            date,
            if clean_path == "index" {
                String::new()
            } else {
                format!("_{}", clean_path)
            }
        ))
    }
}