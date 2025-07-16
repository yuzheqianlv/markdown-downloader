use std::time::Duration;
use tokio::time::sleep;

pub struct ProgressTracker {
    total_pages: usize,
    success_count: usize,
    error_count: usize,
    batch_size: usize,
    wait_time: u64,
    request_delay: u64,
}

impl ProgressTracker {
    pub fn new(total_pages: usize, batch_size: usize, wait_time: u64, request_delay: u64) -> Self {
        Self {
            total_pages,
            success_count: 0,
            error_count: 0,
            batch_size,
            wait_time,
            request_delay,
        }
    }

    pub fn log_start(&self, url: &str) {
        println!("开始爬取网站: {}", url);
        println!("发现 {} 个页面，开始下载", self.total_pages);
        println!("批处理设置: 每 {} 个页面等待 {} 秒", self.batch_size, self.wait_time);
    }

    pub fn log_processing(&self, index: usize, url: &str) {
        println!("[{}/{}] 处理: {}", index + 1, self.total_pages, url);
    }

    pub fn log_success(&mut self, filename: &str) {
        self.success_count += 1;
        println!("✓ 保存为: {}", filename);
    }

    pub fn log_error(&mut self, error: &str) {
        self.error_count += 1;
        eprintln!("✗ 失败: {}", error);
    }

    pub async fn handle_delays(&self, index: usize) {
        // 基本延迟，避免请求过于频繁
        sleep(Duration::from_millis(self.request_delay)).await;

        // 批处理限制：每处理 batch_size 个页面后等待
        if (index + 1) % self.batch_size == 0 && index + 1 < self.total_pages {
            println!(
                "⏳ 已处理 {} 个页面，等待 {} 秒后继续...",
                index + 1,
                self.wait_time
            );
            sleep(Duration::from_secs(self.wait_time)).await;
            println!("🔄 继续处理剩余页面...");
        }
    }

    pub fn log_completion(&self) {
        println!("\n下载完成!");
        println!("成功: {} 个文件", self.success_count);
        println!("失败: {} 个文件", self.error_count);
    }

    pub fn get_stats(&self) -> (usize, usize) {
        (self.success_count, self.error_count)
    }
}