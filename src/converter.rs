use reqwest::Client;
use std::time::Duration;
use crate::config::Config;
use crate::error::Result;

#[derive(Clone)]
pub struct MarkdownConverter {
    client: Client,
    config: Config,
}

impl MarkdownConverter {
    pub fn new(config: Config) -> Self {
        let client = Client::new();
        Self { client, config }
    }

    pub async fn convert_url_to_markdown(&self, url: &str) -> Result<String> {
        let jina_url = format!("https://r.jina.ai/{}", url);

        let response = self
            .client
            .get(&jina_url)
            .header("User-Agent", &self.config.user_agent)
            .timeout(Duration::from_secs(self.config.timeout))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(format!("HTTP错误: {}", response.status()).into());
        }

        let markdown_content = response.text().await?;
        Ok(markdown_content)
    }
}