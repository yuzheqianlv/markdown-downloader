use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct TranslationConfig {
    pub enabled: bool,
    pub source_lang: String,
    pub target_lang: String,
    pub deeplx_api_url: String,
    pub max_requests_per_second: f64,
    pub max_text_length: usize,
    pub max_paragraphs_per_request: usize,
}

impl Default for TranslationConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            source_lang: "auto".to_string(),
            target_lang: "zh".to_string(),
            deeplx_api_url: "http://localhost:1188/translate".to_string(),
            max_requests_per_second: 0.5,  // 大幅降低请求频率
            max_text_length: 3000,  // 保持合理长度
            max_paragraphs_per_request: 10,  // 减少段落数
        }
    }
}

#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: usize,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 2,  // 减少重试次数
            initial_delay_ms: 300,  // 减少初始延迟
            max_delay_ms: 3000,  // 减少最大延迟
            backoff_multiplier: 1.5,  // 减少退避倍数
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeepLXRequest {
    pub text: String,
    pub source_lang: String,
    pub target_lang: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DpTransRequest {
    pub text: String,
    pub source_lang: String,
    pub target_lang: String,
}

#[derive(Debug, Deserialize)]
pub struct DeepLXResponse {
    pub code: i32,
    pub data: String,
}

#[derive(Debug, Clone)]
pub enum FileType {
    Original,
    Translated,
    Bilingual,
}

impl FileType {
    pub fn folder_name(&self) -> &'static str {
        match self {
            FileType::Original => "original",
            FileType::Translated => "translated", 
            FileType::Bilingual => "bilingual",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProcessedLink {
    pub url: String,
    pub processed: bool,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub filename: String,
}

impl ProcessedLink {
    pub fn new(url: String, filename: String) -> Self {
        Self {
            url,
            processed: true,
            timestamp: chrono::Utc::now(),
            filename,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TextSegment {
    pub content: String,
    pub is_code_block: bool,
}