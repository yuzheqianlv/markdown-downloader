use crate::error::Result;
use crate::types::TranslationConfig;
use crate::config_file::ConfigFile;

#[derive(Debug, Clone)]
pub struct Config {
    pub url: String,
    pub output_dir: String,
    pub max_pages: u32,
    pub batch_size: usize,
    pub wait_time: u64,
    pub request_delay: u64,
    pub timeout: u64,
    pub user_agent: String,
    pub translation: TranslationConfig,
}

impl Config {
    pub fn new(
        url: String,
        output_dir: String,
        max_pages: u32,
        batch_size: usize,
        wait_time: u64,
    ) -> Self {
        Self {
            url,
            output_dir,
            max_pages,
            batch_size,
            wait_time,
            request_delay: 500,
            timeout: 30,
            user_agent: "Mozilla/5.0 (compatible; MarkdownDownloader/1.0)".to_string(),
            translation: TranslationConfig::default(),
        }
    }

    pub fn from_config_file(url: String, config_file: &ConfigFile) -> Self {
        Self {
            url,
            output_dir: config_file.general.output_dir.clone(),
            max_pages: config_file.general.max_pages,
            batch_size: config_file.general.batch_size,
            wait_time: config_file.general.wait_time,
            request_delay: config_file.general.request_delay,
            timeout: config_file.general.timeout,
            user_agent: config_file.general.user_agent.clone(),
            translation: config_file.to_translation_config(),
        }
    }

    pub fn with_translation(mut self, translation: TranslationConfig) -> Self {
        self.translation = translation;
        self
    }

    pub fn validate(&self) -> Result<()> {
        if self.url.is_empty() {
            return Err("URL cannot be empty".into());
        }
        
        if self.output_dir.is_empty() {
            return Err("Output directory cannot be empty".into());
        }
        
        if self.max_pages == 0 {
            return Err("Max pages must be greater than 0".into());
        }
        
        if self.batch_size == 0 {
            return Err("Batch size must be greater than 0".into());
        }
        
        Ok(())
    }
}