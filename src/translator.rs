use crate::types::{TranslationConfig, DeepLXRequest, DeepLXResponse, DpTransRequest, RetryConfig, TextSegment};
use crate::error::Result;
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::time::sleep;

#[derive(Clone)]
pub struct RateLimiter {
    semaphore: Arc<Semaphore>,
    delay: Duration,
}

impl RateLimiter {
    pub fn new(requests_per_second: f64) -> Self {
        let permits = requests_per_second.ceil() as usize;
        let delay = Duration::from_millis((1000.0 / requests_per_second) as u64);

        Self {
            semaphore: Arc::new(Semaphore::new(permits)),
            delay,
        }
    }

    pub async fn acquire(&self) -> Result<()> {
        let _permit = self.semaphore.acquire().await
            .map_err(|e| crate::error::Error::Custom(format!("Rate limiter error: {}", e)))?;
        sleep(self.delay).await;
        Ok(())
    }
}

pub async fn retry_with_backoff<F, Fut, T>(
    mut operation: F,
    config: &RetryConfig,
    rate_limiter: &RateLimiter,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let mut delay = config.initial_delay_ms;

    for attempt in 0..=config.max_retries {
        rate_limiter.acquire().await?;

        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt == config.max_retries => return Err(e),
            Err(e) => {
                eprintln!("Attempt {} failed: {}. Retrying in {}ms...", attempt + 1, e, delay);
                sleep(Duration::from_millis(delay)).await;
                delay = std::cmp::min(
                    (delay as f64 * config.backoff_multiplier) as u64,
                    config.max_delay_ms,
                );
            }
        }
    }

    unreachable!()
}

#[derive(Clone)]
pub struct TranslationService {
    client: Client,
    rate_limiter: RateLimiter,
    config: TranslationConfig,
}

impl TranslationService {
    pub fn new(config: TranslationConfig) -> Self {
        // 创建兼容性更好的 HTTP 客户端
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(30))  // 增加超时时间
            .pool_idle_timeout(std::time::Duration::from_secs(30))
            .pool_max_idle_per_host(5)  // 减少连接池大小
            .tcp_keepalive(std::time::Duration::from_secs(60))
            .http1_title_case_headers()  // 强制使用 HTTP/1.1
            .http2_keep_alive_interval(None)  // 禁用 HTTP/2 keep-alive
            .user_agent("Mozilla/5.0 (compatible; MarkdownDownloader/1.0)")
            .build()
            .unwrap_or_else(|e| {
                eprintln!("Failed to create optimized client: {}, using default", e);
                Client::new()
            });
            
        Self {
            client,
            rate_limiter: RateLimiter::new(config.max_requests_per_second),
            config,
        }
    }

    pub async fn translate(&self, text: &str) -> Result<String> {
        if !self.config.enabled {
            return Ok(text.to_string());
        }

        println!("文本总长度: {} 字符", text.len());

        if text.len() <= self.config.max_text_length {
            println!("文本较短，直接翻译");
            return self.translate_chunk(text).await;
        }

        let chunks = self.split_text_into_chunks(text);
        println!("文本较长，分为 {} 块进行翻译", chunks.len());

        let mut translated_chunks = Vec::new();

        for (i, chunk) in chunks.iter().enumerate() {
            println!("翻译第 {} 块，长度: {} 字符", i + 1, chunk.len());

            let translated_chunk = self.translate_chunk(chunk).await?;
            translated_chunks.push(translated_chunk);
        }

        Ok(translated_chunks.join("\n\n"))
    }

    fn split_text_into_chunks(&self, text: &str) -> Vec<String> {
        let mut chunks = Vec::new();
        let max_length = self.config.max_text_length;

        if text.len() <= max_length {
            chunks.push(text.to_string());
            return chunks;
        }

        // 识别和保护代码块
        let protected_sections = self.identify_code_blocks(text);
        let segments = self.split_by_code_blocks(text, &protected_sections);

        let mut current_chunk = String::new();

        for segment in segments {
            if segment.is_code_block {
                // 代码块作为完整单元处理
                if !current_chunk.is_empty() && current_chunk.len() + segment.content.len() + 2 > max_length {
                    // 当前块加上代码块会超长，先保存当前块
                    chunks.push(current_chunk.clone());
                    current_chunk.clear();
                }
                
                if segment.content.len() > max_length {
                    // 如果代码块本身超长，但仍作为整体翻译
                    if !current_chunk.is_empty() {
                        chunks.push(current_chunk.clone());
                        current_chunk.clear();
                    }
                    chunks.push(segment.content);
                } else {
                    if !current_chunk.is_empty() {
                        current_chunk.push_str("\n\n");
                    }
                    current_chunk.push_str(&segment.content);
                }
            } else {
                // 普通文本按段落分割
                let paragraphs = self.split_text_by_empty_lines(&segment.content);
                
                for paragraph in paragraphs {
                    if paragraph.trim().is_empty() {
                        continue;
                    }

                    let potential_length = if current_chunk.is_empty() {
                        paragraph.len()
                    } else {
                        current_chunk.len() + 2 + paragraph.len()
                    };

                    if potential_length <= max_length {
                        if !current_chunk.is_empty() {
                            current_chunk.push_str("\n\n");
                        }
                        current_chunk.push_str(&paragraph);
                    } else {
                        if !current_chunk.is_empty() {
                            chunks.push(current_chunk.clone());
                            current_chunk.clear();
                        }

                        if paragraph.len() > max_length {
                            let sub_chunks = self.split_long_paragraph(&paragraph, max_length);
                            chunks.extend(sub_chunks);
                        } else {
                            current_chunk = paragraph;
                        }
                    }
                }
            }
        }

        if !current_chunk.is_empty() {
            chunks.push(current_chunk);
        }

        if chunks.is_empty() {
            chunks.push(text.to_string());
        }

        chunks
    }

    // 识别代码块的位置
    fn identify_code_blocks(&self, text: &str) -> Vec<(usize, usize)> {
        let mut code_blocks = Vec::new();
        let mut in_code_block = false;
        let mut current_start = 0;
        
        let lines: Vec<&str> = text.lines().collect();
        let mut char_pos = 0;
        
        for (i, line) in lines.iter().enumerate() {
            if line.starts_with("```") {
                if in_code_block {
                    // 代码块结束
                    let end_pos = char_pos + line.len();
                    code_blocks.push((current_start, end_pos));
                    in_code_block = false;
                } else {
                    // 代码块开始
                    current_start = char_pos;
                    in_code_block = true;
                }
            }
            char_pos += line.len() + 1; // +1 for newline
        }
        
        // 如果有未闭合的代码块，将其延伸到文本结尾
        if in_code_block {
            code_blocks.push((current_start, text.len()));
        }
        
        code_blocks
    }

    // 根据代码块分割文本
    fn split_by_code_blocks(&self, text: &str, code_blocks: &[(usize, usize)]) -> Vec<TextSegment> {
        let mut segments = Vec::new();
        let mut last_end = 0;
        
        for &(start, end) in code_blocks {
            // 添加代码块前的普通文本
            if start > last_end {
                let content = text[last_end..start].to_string();
                if !content.trim().is_empty() {
                    segments.push(TextSegment {
                        content,
                        is_code_block: false,
                    });
                }
            }
            
            // 添加代码块
            let content = text[start..end].to_string();
            segments.push(TextSegment {
                content,
                is_code_block: true,
            });
            
            last_end = end;
        }
        
        // 添加最后剩余的普通文本
        if last_end < text.len() {
            let content = text[last_end..].to_string();
            if !content.trim().is_empty() {
                segments.push(TextSegment {
                    content,
                    is_code_block: false,
                });
            }
        }
        
        // 如果没有代码块，整个文本作为普通文本
        if segments.is_empty() {
            segments.push(TextSegment {
                content: text.to_string(),
                is_code_block: false,
            });
        }
        
        segments
    }

    // 根据空行分割文本，但更智能地选择分割点
    fn split_text_by_empty_lines(&self, text: &str) -> Vec<String> {
        let max_length = self.config.max_text_length;
        
        // 如果文本不长，直接返回
        if text.len() <= max_length {
            return vec![text.to_string()];
        }
        
        // 按双换行符分割得到段落
        let paragraphs: Vec<&str> = text.split("\n\n").collect();
        let mut result = Vec::new();
        let mut current_group = Vec::new();
        let mut current_length = 0;
        
        for paragraph in paragraphs {
            let paragraph = paragraph.trim();
            if paragraph.is_empty() {
                continue;
            }
            
            let para_len = paragraph.len();
            
            // 检查加入当前段落后是否超长
            let potential_length = if current_group.is_empty() {
                para_len
            } else {
                current_length + 2 + para_len // +2 for "\n\n"
            };
            
            if potential_length <= max_length {
                // 可以加入当前组
                current_group.push(paragraph);
                current_length = potential_length;
            } else {
                // 超长了，先保存当前组
                if !current_group.is_empty() {
                    result.push(current_group.join("\n\n"));
                    current_group.clear();
                }
                
                // 检查单个段落是否超长
                if para_len > max_length {
                    // 需要进一步分割
                    let sub_parts = self.split_long_paragraph(paragraph, max_length);
                    result.extend(sub_parts);
                    current_length = 0;
                } else {
                    // 作为新组的开始
                    current_group.push(paragraph);
                    current_length = para_len;
                }
            }
        }
        
        // 处理最后一组
        if !current_group.is_empty() {
            result.push(current_group.join("\n\n"));
        }
        
        result
    }

    fn split_long_paragraph(&self, paragraph: &str, max_length: usize) -> Vec<String> {
        let mut chunks = Vec::new();
        let mut start = 0;

        while start < paragraph.len() {
            let end = std::cmp::min(start + max_length, paragraph.len());
            let mut actual_end = end;

            if end < paragraph.len() {
                for i in (start..end).rev() {
                    let ch = paragraph.chars().nth(i).unwrap_or(' ');
                    if ch == '.' || ch == '!' || ch == '?' || ch == '。' || ch == '！' || ch == '？' {
                        actual_end = i + 1;
                        break;
                    }
                }

                if actual_end == end {
                    for i in (start..end).rev() {
                        let ch = paragraph.chars().nth(i).unwrap_or(' ');
                        if ch == ' ' || ch == '\n' || ch == '\t' {
                            actual_end = i + 1;
                            break;
                        }
                    }
                }

                if actual_end == end && end - start < max_length / 2 {
                    actual_end = end;
                }
            }

            let chunk = paragraph[start..actual_end].trim().to_string();
            if !chunk.is_empty() {
                chunks.push(chunk);
            }

            start = actual_end;
        }

        chunks
    }

    async fn translate_chunk(&self, text: &str) -> Result<String> {
        println!("发送翻译请求到: {}", self.config.deeplx_api_url);
        println!("翻译文本长度: {} 字符", text.len());

        let retry_config = RetryConfig::default();
        let client = &self.client;
        let config = &self.config;
        let text_clone = text.to_string();

        let result = retry_with_backoff(
            || {
                let client = client.clone();
                let config = config.clone();
                let text = text_clone.clone();

                Box::pin(async move {
                    let response = if config.deeplx_api_url.contains("dptrans") {
                        println!("使用dptrans API格式请求");

                        let request = DpTransRequest {
                            text: text.clone(),
                            source_lang: if config.source_lang == "auto" { "auto".to_string() } else { config.source_lang.clone() },
                            target_lang: config.target_lang.clone(),
                        };

                        client
                            .post(&config.deeplx_api_url)
                            .header("Content-Type", "application/json")
                            .header("Accept", "application/json, text/plain, */*")
                            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                            .json(&request)
                            .send()
                            .await
                            .map_err(|e| {
                                crate::error::Error::Custom(format!("DeepLX网络请求失败: {}", e))
                            })?
                    } else {
                        println!("使用标准DeepLX API格式请求");

                        let request = DeepLXRequest {
                            text: text.clone(),
                            source_lang: config.source_lang.clone(),
                            target_lang: config.target_lang.clone(),
                        };

                        client
                            .post(&config.deeplx_api_url)
                            .header("Content-Type", "application/json")
                            .header("Accept", "application/json")
                            .json(&request)
                            .send()
                            .await
                            .map_err(|e| {
                                crate::error::Error::Custom(format!("DeepLX网络请求失败: {}", e))
                            })?
                    };

                    let status = response.status();
                    println!("DeepLX响应状态: {}", status);

                    if response.status().is_success() {
                        let response_text = response
                            .text()
                            .await
                            .map_err(|e| crate::error::Error::Custom(format!("读取响应文本失败: {}", e)))?;

                        if let Ok(result) = serde_json::from_str::<DeepLXResponse>(&response_text) {
                            if result.code == 200 {
                                if result.data.is_empty() {
                                    Err(crate::error::Error::Custom("DeepLX返回了空的翻译结果".to_string()))
                                } else {
                                    Ok(result.data)
                                }
                            } else {
                                Err(crate::error::Error::Custom(format!(
                                    "DeepLX翻译失败，返回代码: {}",
                                    result.code
                                )))
                            }
                        } else {
                            if response_text.trim().is_empty() {
                                Err(crate::error::Error::Custom("API返回了空的翻译结果".to_string()))
                            } else if response_text.starts_with("{") {
                                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&response_text) {
                                    if let Some(translated) = json_value
                                        .get("translated_text")
                                        .or_else(|| json_value.get("result"))
                                        .or_else(|| json_value.get("translation"))
                                        .or_else(|| json_value.get("data"))
                                        .and_then(|v| v.as_str())
                                    {
                                        Ok(translated.to_string())
                                    } else {
                                        Err(crate::error::Error::Custom(format!(
                                            "无法从JSON响应中提取翻译结果: {}",
                                            response_text
                                        )))
                                    }
                                } else {
                                    Err(crate::error::Error::Custom(format!("无法解析JSON响应: {}", response_text)))
                                }
                            } else {
                                println!("假设响应是纯文本翻译结果");
                                Ok(response_text)
                            }
                        }
                    } else {
                        let error_text = response
                            .text()
                            .await
                            .unwrap_or_else(|_| "无法读取错误信息".to_string());
                        Err(crate::error::Error::Custom(format!(
                            "DeepLX API请求失败: {} - {}",
                            status, error_text
                        )))
                    }
                })
            },
            &retry_config,
            &self.rate_limiter,
        )
        .await?;

        Ok(result)
    }
}