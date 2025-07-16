use crate::error::Result;
use std::collections::HashSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use url::Url;

pub struct LinksManager {
    links_file_path: PathBuf,
    processed_links: HashSet<String>,
}

impl LinksManager {
    pub fn new(base_dir: &Path, url: &str) -> Result<Self> {
        let domain = Self::extract_domain(url)?;
        let links_file_name = format!("{}_links.txt", domain.replace('.', "_"));
        let mut links_file_path = base_dir.to_path_buf();
        links_file_path.push(links_file_name);

        let mut manager = Self {
            links_file_path,
            processed_links: HashSet::new(),
        };

        manager.load_processed_links()?;
        Ok(manager)
    }

    pub fn is_processed(&self, url: &str) -> bool {
        self.processed_links.contains(url)
    }

    pub fn mark_as_processed(&mut self, url: &str, filename: &str) -> Result<()> {
        if self.processed_links.insert(url.to_string()) {
            self.append_to_file(url, filename)?;
        }
        Ok(())
    }

    pub fn mark_as_failed(&mut self, url: &str, error: &str) -> Result<()> {
        let entry = format!("❌ {} | ERROR: {}", url, error);
        self.append_raw_line(&entry)?;
        Ok(())
    }

    pub fn get_processed_count(&self) -> usize {
        self.processed_links.len()
    }

    pub fn get_all_processed_links(&self) -> Vec<String> {
        self.processed_links.iter().cloned().collect()
    }

    fn load_processed_links(&mut self) -> Result<()> {
        if !self.links_file_path.exists() {
            self.create_links_file()?;
            return Ok(());
        }

        let content = fs::read_to_string(&self.links_file_path)?;
        
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            if line.starts_with("✅") {
                if let Some(url) = self.extract_url_from_line(line) {
                    self.processed_links.insert(url);
                }
            }
        }

        println!("从 {} 加载了 {} 个已处理的链接", 
            self.links_file_path.display(), 
            self.processed_links.len()
        );

        Ok(())
    }

    fn create_links_file(&self) -> Result<()> {
        let header = format!(
            "# Links Processing Record\n# Generated on: {}\n# Format: ✅ URL | filename\n# ❌ URL | ERROR: error_message\n\n",
            chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
        );
        
        fs::write(&self.links_file_path, header)?;
        println!("创建链接记录文件: {}", self.links_file_path.display());
        Ok(())
    }

    fn append_to_file(&self, url: &str, filename: &str) -> Result<()> {
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S");
        let entry = format!("✅ {} | {} | {}\n", url, filename, timestamp);
        
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.links_file_path)?
            .write_all(entry.as_bytes())?;
        
        Ok(())
    }

    fn append_raw_line(&self, line: &str) -> Result<()> {
        let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S");
        let entry = format!("{} | {}\n", line, timestamp);
        
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.links_file_path)?
            .write_all(entry.as_bytes())?;
        
        Ok(())
    }

    fn extract_url_from_line(&self, line: &str) -> Option<String> {
        if let Some(start) = line.find(' ') {
            let rest = &line[start + 1..];
            if let Some(end) = rest.find(" | ") {
                return Some(rest[..end].trim().to_string());
            }
        }
        None
    }

    fn extract_domain(url: &str) -> Result<String> {
        let parsed_url = Url::parse(url)
            .map_err(|e| crate::error::Error::Custom(format!("Invalid URL: {}", e)))?;
        
        let domain = parsed_url.host_str()
            .ok_or_else(|| crate::error::Error::Custom("No host in URL".to_string()))?;
        
        Ok(domain.to_string())
    }

    pub fn print_summary(&self) {
        println!("\n📊 处理摘要:");
        println!("   已处理链接: {}", self.processed_links.len());
        println!("   记录文件: {}", self.links_file_path.display());
    }

    pub fn filter_unprocessed_urls(&self, urls: Vec<String>) -> Vec<String> {
        urls.into_iter()
            .filter(|url| !self.is_processed(url))
            .collect()
    }

    pub fn export_processed_links(&self, export_path: &Path) -> Result<()> {
        let mut export_content = String::new();
        export_content.push_str("# Processed Links Export\n");
        export_content.push_str(&format!("# Exported on: {}\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
        export_content.push_str(&format!("# Total links: {}\n\n", self.processed_links.len()));

        for url in &self.processed_links {
            export_content.push_str(&format!("{}\n", url));
        }

        fs::write(export_path, export_content)?;
        println!("导出已处理链接到: {}", export_path.display());
        Ok(())
    }
}