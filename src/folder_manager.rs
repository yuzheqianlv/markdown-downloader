use crate::types::{FileType, TranslationConfig};
use crate::error::Result;
use std::fs;
use std::path::{Path, PathBuf};
use url::Url;

#[derive(Clone)]
pub struct FolderManager {
    base_dir: PathBuf,
    translation_config: TranslationConfig,
}

impl FolderManager {
    pub fn new(base_dir: PathBuf, translation_config: TranslationConfig) -> Self {
        Self {
            base_dir,
            translation_config,
        }
    }

    pub fn get_folder_path(&self, url: &str, file_type: FileType) -> Result<PathBuf> {
        let domain = self.extract_domain(url)?;
        
        let folder_name = if self.translation_config.enabled {
            format!("{}_{}-{}", 
                domain, 
                self.translation_config.source_lang, 
                self.translation_config.target_lang
            )
        } else {
            domain
        };

        let mut path = self.base_dir.clone();
        path.push(folder_name);
        
        if self.translation_config.enabled {
            path.push(file_type.folder_name());
        }

        Ok(path)
    }

    pub fn ensure_folder_exists(&self, folder_path: &Path) -> Result<()> {
        if !folder_path.exists() {
            fs::create_dir_all(folder_path)?;
            println!("创建文件夹: {}", folder_path.display());
        }
        Ok(())
    }

    pub fn create_all_folders(&self, url: &str) -> Result<()> {
        if self.translation_config.enabled {
            for file_type in [FileType::Original, FileType::Translated, FileType::Bilingual] {
                let folder_path = self.get_folder_path(url, file_type)?;
                self.ensure_folder_exists(&folder_path)?;
            }
        } else {
            let folder_path = self.get_folder_path(url, FileType::Original)?;
            self.ensure_folder_exists(&folder_path)?;
        }
        Ok(())
    }

    pub fn get_file_path(&self, url: &str, file_type: FileType) -> Result<PathBuf> {
        let folder_path = self.get_folder_path(url, file_type)?;
        self.ensure_folder_exists(&folder_path)?;
        
        let filename = self.generate_filename(url)?;
        let mut file_path = folder_path;
        file_path.push(filename);
        
        Ok(file_path)
    }

    fn extract_domain(&self, url: &str) -> Result<String> {
        let parsed_url = Url::parse(url)
            .map_err(|e| crate::error::Error::Custom(format!("Invalid URL: {}", e)))?;
        
        let domain = parsed_url.host_str()
            .ok_or_else(|| crate::error::Error::Custom("No host in URL".to_string()))?;
        
        Ok(domain.to_string())
    }

    fn generate_filename(&self, url: &str) -> Result<String> {
        let parsed_url = Url::parse(url)
            .map_err(|e| crate::error::Error::Custom(format!("Invalid URL: {}", e)))?;

        let host = parsed_url.host_str().unwrap_or("unknown");
        let path = parsed_url.path();
        
        let date = chrono::Utc::now().format("%Y%m%d").to_string();
        
        let path_part = if path == "/" || path.is_empty() {
            "index".to_string()
        } else {
            path.trim_start_matches('/')
                .replace('/', "_")
                .replace(['?', '#', '&', '=', '%'], "_")
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '-' || *c == '.')
                .collect::<String>()
                .trim_end_matches('.')
                .to_string()
        };

        let filename = format!("{}_{}_{}_{}.md", 
            host.replace('.', "_"), 
            date,
            if path_part.len() > 100 { 
                format!("{}...", &path_part[..97])
            } else { 
                path_part 
            },
            chrono::Utc::now().timestamp()
        );

        Ok(filename)
    }

    pub fn create_bilingual_content(&self, original: &str, translated: &str) -> String {
        let mut bilingual = String::new();
        
        // 按段落分割（双换行符）
        let original_paragraphs: Vec<&str> = original.split("\n\n").collect();
        let translated_paragraphs: Vec<&str> = translated.split("\n\n").collect();
        
        // 如果段落数量匹配，采用段落对应模式
        if original_paragraphs.len() == translated_paragraphs.len() {
            for (orig_para, trans_para) in original_paragraphs.iter().zip(translated_paragraphs.iter()) {
                let orig_para = orig_para.trim();
                let trans_para = trans_para.trim();
                
                if !orig_para.is_empty() || !trans_para.is_empty() {
                    // 译文在上
                    if !trans_para.is_empty() {
                        bilingual.push_str(trans_para);
                        bilingual.push_str("\n\n");
                    }
                    
                    // 原文在下
                    if !orig_para.is_empty() {
                        bilingual.push_str(orig_para);
                        bilingual.push_str("\n\n");
                    }
                    
                    // 段落间额外空行
                    bilingual.push('\n');
                }
            }
        } else {
            // 段落数量不匹配，采用整体对比模式
            let original = original.trim();
            let translated = translated.trim();
            
            if !translated.is_empty() {
                bilingual.push_str("**译文:**\n\n");
                bilingual.push_str(translated);
                bilingual.push_str("\n\n---\n\n");
            }
            
            if !original.is_empty() {
                bilingual.push_str("**原文:**\n\n");
                bilingual.push_str(original);
                bilingual.push('\n');
            }
        }
        
        bilingual
    }

    pub fn save_content(&self, url: &str, original: &str, translated: Option<&str>) -> Result<Vec<String>> {
        let mut saved_files = Vec::new();

        if self.translation_config.enabled {
            let original_path = self.get_file_path(url, FileType::Original)?;
            fs::write(&original_path, original)?;
            saved_files.push(original_path.to_string_lossy().to_string());
            println!("保存原文: {}", original_path.display());

            if let Some(translated_content) = translated {
                let translated_path = self.get_file_path(url, FileType::Translated)?;
                fs::write(&translated_path, translated_content)?;
                saved_files.push(translated_path.to_string_lossy().to_string());
                println!("保存译文: {}", translated_path.display());

                let bilingual_content = self.create_bilingual_content(original, translated_content);
                let bilingual_path = self.get_file_path(url, FileType::Bilingual)?;
                fs::write(&bilingual_path, bilingual_content)?;
                saved_files.push(bilingual_path.to_string_lossy().to_string());
                println!("保存双语对照: {}", bilingual_path.display());
            }
        } else {
            let file_path = self.get_file_path(url, FileType::Original)?;
            fs::write(&file_path, original)?;
            saved_files.push(file_path.to_string_lossy().to_string());
            println!("保存文件: {}", file_path.display());
        }

        Ok(saved_files)
    }
}