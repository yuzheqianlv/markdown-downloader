use crate::types::TranslationConfig;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigFile {
    #[serde(default)]
    pub general: GeneralConfig,
    #[serde(default)]
    pub translation: TranslationFileConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_output_dir")]
    pub output_dir: String,
    #[serde(default = "default_max_pages")]
    pub max_pages: u32,
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
    #[serde(default = "default_wait_time")]
    pub wait_time: u64,
    #[serde(default = "default_request_delay")]
    pub request_delay: u64,
    #[serde(default = "default_timeout")]
    pub timeout: u64,
    #[serde(default = "default_user_agent")]
    pub user_agent: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationFileConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_source_lang")]
    pub source_lang: String,
    #[serde(default = "default_target_lang")]
    pub target_lang: String,
    #[serde(default = "default_deeplx_url")]
    pub deeplx_api_url: String,
    #[serde(default = "default_max_requests_per_second")]
    pub max_requests_per_second: f64,
    #[serde(default = "default_max_text_length")]
    pub max_text_length: usize,
    #[serde(default = "default_max_paragraphs_per_request")]
    pub max_paragraphs_per_request: usize,
}

// Default value functions
fn default_output_dir() -> String { "./downloads".to_string() }
fn default_max_pages() -> u32 { 50 }
fn default_batch_size() -> usize { 10 }
fn default_wait_time() -> u64 { 60 }
fn default_request_delay() -> u64 { 500 }
fn default_timeout() -> u64 { 30 }
fn default_user_agent() -> String { "Mozilla/5.0 (compatible; MarkdownDownloader/1.0)".to_string() }
fn default_source_lang() -> String { "auto".to_string() }
fn default_target_lang() -> String { "zh".to_string() }
fn default_deeplx_url() -> String { "http://localhost:1188/translate".to_string() }
fn default_max_requests_per_second() -> f64 { 2.0 }
fn default_max_text_length() -> usize { 3000 }
fn default_max_paragraphs_per_request() -> usize { 10 }

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            output_dir: default_output_dir(),
            max_pages: default_max_pages(),
            batch_size: default_batch_size(),
            wait_time: default_wait_time(),
            request_delay: default_request_delay(),
            timeout: default_timeout(),
            user_agent: default_user_agent(),
        }
    }
}

impl Default for TranslationFileConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            source_lang: default_source_lang(),
            target_lang: default_target_lang(),
            deeplx_api_url: default_deeplx_url(),
            max_requests_per_second: default_max_requests_per_second(),
            max_text_length: default_max_text_length(),
            max_paragraphs_per_request: default_max_paragraphs_per_request(),
        }
    }
}

impl Default for ConfigFile {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            translation: TranslationFileConfig::default(),
        }
    }
}

impl ConfigFile {
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: ConfigFile = toml::from_str(&content)
            .map_err(|e| crate::error::Error::Custom(format!("Failed to parse config file: {}", e)))?;
        Ok(config)
    }

    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| crate::error::Error::Custom(format!("Failed to serialize config: {}", e)))?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn find_config_file() -> Option<PathBuf> {
        let possible_paths = vec![
            PathBuf::from("markdown-downloader.toml"),
            PathBuf::from("config.toml"),
            PathBuf::from(".markdown-downloader.toml"),
        ];

        for path in possible_paths {
            if path.exists() {
                return Some(path);
            }
        }

        // Check in home directory
        if let Some(home) = dirs::home_dir() {
            let home_config = home.join(".config").join("markdown-downloader").join("config.toml");
            if home_config.exists() {
                return Some(home_config);
            }
        }

        None
    }

    pub fn load_default() -> Self {
        if let Some(config_path) = Self::find_config_file() {
            match Self::load_from_file(&config_path) {
                Ok(config) => {
                    println!("已加载配置文件: {}", config_path.display());
                    return config;
                }
                Err(e) => {
                    eprintln!("加载配置文件失败 {}: {}", config_path.display(), e);
                    eprintln!("使用默认配置");
                }
            }
        }
        Self::default()
    }

    pub fn to_translation_config(&self) -> TranslationConfig {
        TranslationConfig {
            enabled: self.translation.enabled,
            source_lang: self.translation.source_lang.clone(),
            target_lang: self.translation.target_lang.clone(),
            deeplx_api_url: self.translation.deeplx_api_url.clone(),
            max_requests_per_second: self.translation.max_requests_per_second,
            max_text_length: self.translation.max_text_length,
            max_paragraphs_per_request: self.translation.max_paragraphs_per_request,
        }
    }

    pub fn create_example_config<P: AsRef<Path>>(path: P) -> Result<()> {
        let example_config = ConfigFile {
            general: GeneralConfig {
                output_dir: "./downloads".to_string(),
                max_pages: 50,
                batch_size: 10,
                wait_time: 60,
                request_delay: 500,
                timeout: 30,
                user_agent: "Mozilla/5.0 (compatible; MarkdownDownloader/1.0)".to_string(),
            },
            translation: TranslationFileConfig {
                enabled: false,
                source_lang: "auto".to_string(),
                target_lang: "zh".to_string(),
                deeplx_api_url: "http://localhost:1188/translate".to_string(),
                max_requests_per_second: 2.0,
                max_text_length: 3000,
                max_paragraphs_per_request: 10,
            },
        };

        example_config.save_to_file(path)?;
        Ok(())
    }
}