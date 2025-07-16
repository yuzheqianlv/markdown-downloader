use spider::website::Website;
use crate::config::Config;
use crate::error::Result;

pub struct Crawler {
    website: Website,
}

impl Crawler {
    pub fn new(config: &Config) -> Result<Self> {
        let website = Website::new(&config.url)
            .with_respect_robots_txt(true)
            .with_delay(1000) // 1秒延迟
            .with_user_agent(Some(&config.user_agent))
            .with_limit(config.max_pages)
            .build()
            .map_err(|e| crate::error::Error::Spider(Box::new(e)))?;

        Ok(Self { website })
    }

    pub async fn crawl(&mut self) -> Result<Vec<String>> {
        self.website.scrape().await;
        
        let links = self.website.get_links();
        let urls: Vec<String> = links.iter().map(|link| link.as_ref().to_string()).collect();
        
        Ok(urls)
    }
}