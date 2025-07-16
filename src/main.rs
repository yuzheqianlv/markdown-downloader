use markdown_downloader::{
    cli, Crawler, MarkdownConverter, FileManager, ProgressTracker, 
    TranslationService, FolderManager, LinksManager, Result
};

#[tokio::main]
async fn main() -> Result<()> {
    // 解析命令行参数
    let config = cli::parse_args()?;

    // 初始化组件
    let mut crawler = Crawler::new(&config)?;
    let converter = MarkdownConverter::new(config.clone());
    let _file_manager = FileManager::new(config.output_dir.clone())?;
    
    // 初始化翻译相关组件
    let translation_service = if config.translation.enabled {
        Some(TranslationService::new(config.translation.clone()))
    } else {
        None
    };
    
    let folder_manager = FolderManager::new(
        std::path::PathBuf::from(&config.output_dir),
        config.translation.clone()
    );
    
    let links_manager = LinksManager::new(
        std::path::Path::new(&config.output_dir),
        &config.url
    )?;

    // 爬取网站
    let all_urls = crawler.crawl().await?;
    
    // 过滤未处理的链接
    let urls = links_manager.filter_unprocessed_urls(all_urls);
    let total_pages = urls.len();
    
    println!("发现 {} 个新链接需要处理", total_pages);
    
    // 创建文件夹结构
    if !urls.is_empty() {
        folder_manager.create_all_folders(&urls[0])?;
    }

    // 初始化进度追踪器
    let progress = ProgressTracker::new(
        total_pages,
        config.batch_size,
        config.wait_time,
        config.request_delay,
    );

    progress.log_start(&config.url);

    // 并发处理URL
    use futures::stream::{self, StreamExt};
    use std::sync::Arc;
    use tokio::sync::Mutex;
    
    let links_manager = Arc::new(Mutex::new(links_manager));
    let progress = Arc::new(Mutex::new(progress));
    
    // 限制并发数量，基于429错误经验进一步保守设置
    let concurrency = std::cmp::min(config.batch_size, 3);
    
    // 分批处理以避免API压力
    let mut processed = 0;
    for batch in urls.chunks(config.batch_size) {
        if processed > 0 {
            println!("等待 {} 秒后继续处理下一批...", config.wait_time);
            tokio::time::sleep(tokio::time::Duration::from_secs(config.wait_time)).await;
        }
        
        stream::iter(batch.iter().enumerate())
            .map(|(batch_index, url)| {
                let index = processed + batch_index;
                let converter = converter.clone();
                let translation_service = translation_service.clone();
                let folder_manager = folder_manager.clone();
                let links_manager = links_manager.clone();
                let progress = progress.clone();
                let config = config.clone();
                let url = url.clone();
                
                async move {
                    {
                        let progress_guard = progress.lock().await;
                        progress_guard.log_processing(index, &url);
                    }
                    
                    // 转换为 Markdown
                    let markdown_result = converter.convert_url_to_markdown(&url).await;
                    
                    match markdown_result {
                        Ok(markdown_content) => {
                            let mut translated_content = None;
                            
                            // 如果启用翻译，进行翻译
                            if let Some(translator) = &translation_service {
                                match translator.translate(&markdown_content).await {
                                    Ok(translated) => {
                                        translated_content = Some(translated);
                                        println!("翻译完成: {}", url);
                                    }
                                    Err(e) => {
                                        eprintln!("翻译失败: {} - {}", url, e);
                                        let mut links_guard = links_manager.lock().await;
                                        let _ = links_guard.mark_as_failed(&url, &format!("Translation failed: {}", e));
                                        let mut progress_guard = progress.lock().await;
                                        progress_guard.log_error(&e.to_string());
                                        return;
                                    }
                                }
                            }

                            // 保存文件
                            match folder_manager.save_content(&url, &markdown_content, translated_content.as_deref()) {
                                Ok(saved_files) => {
                                    let filename = saved_files.first().unwrap_or(&"unknown".to_string()).clone();
                                    let mut links_guard = links_manager.lock().await;
                                    let _ = links_guard.mark_as_processed(&url, &filename);
                                    let mut progress_guard = progress.lock().await;
                                    progress_guard.log_success(&format!("Saved {} files", saved_files.len()));
                                }
                                Err(e) => {
                                    let mut links_guard = links_manager.lock().await;
                                    let _ = links_guard.mark_as_failed(&url, &e.to_string());
                                    let mut progress_guard = progress.lock().await;
                                    progress_guard.log_error(&e.to_string());
                                }
                            }
                        }
                        Err(e) => {
                            let mut links_guard = links_manager.lock().await;
                            let _ = links_guard.mark_as_failed(&url, &e.to_string());
                            let mut progress_guard = progress.lock().await;
                            progress_guard.log_error(&e.to_string());
                        }
                    }
                    
                    // 添加请求间延迟
                    tokio::time::sleep(tokio::time::Duration::from_millis(config.request_delay)).await;
                }
            })
            .buffer_unordered(concurrency)
            .collect::<Vec<_>>()
            .await;
            
        processed += batch.len();
    }

    {
        let progress_guard = progress.lock().await;
        progress_guard.log_completion();
    }
    
    {
        let links_guard = links_manager.lock().await;
        links_guard.print_summary();
    }
    
    Ok(())
}
