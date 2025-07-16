use clap::{Arg, Command};
use crate::config::Config;
use crate::types::TranslationConfig;
use crate::config_file::ConfigFile;
use crate::error::Result;

pub fn parse_args() -> Result<Config> {
    let matches = Command::new("markdown-downloader")
        .version("1.0")
        .about("Download website content as markdown files")
        .arg(
            Arg::new("url")
                .help("Target website URL")
                .required(false)
                .index(1),
        )
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .help("Output directory")
                .default_value("./downloads"),
        )
        .arg(
            Arg::new("max-pages")
                .short('m')
                .long("max-pages")
                .help("Maximum pages to crawl")
                .default_value("50"),
        )
        .arg(
            Arg::new("batch-size")
                .short('b')
                .long("batch-size")
                .help("Number of pages to process before waiting")
                .default_value("10"),
        )
        .arg(
            Arg::new("wait-time")
                .short('w')
                .long("wait-time")
                .help("Wait time in seconds between batches")
                .default_value("60"),
        )
        .arg(
            Arg::new("translate")
                .short('t')
                .long("translate")
                .help("Enable translation")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("source-lang")
                .long("source-lang")
                .help("Source language for translation"),
        )
        .arg(
            Arg::new("target-lang")
                .long("target-lang")
                .help("Target language for translation"),
        )
        .arg(
            Arg::new("deeplx-url")
                .long("deeplx-url")
                .help("DeepLX API URL"),
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .help("Path to configuration file")
                .value_name("FILE"),
        )
        .arg(
            Arg::new("generate-config")
                .long("generate-config")
                .help("Generate example configuration file")
                .value_name("FILE")
                .action(clap::ArgAction::Set),
        )
        .get_matches();

    // 处理生成配置文件的情况
    if let Some(config_path) = matches.get_one::<String>("generate-config") {
        ConfigFile::create_example_config(config_path)?;
        println!("已生成示例配置文件: {}", config_path);
        std::process::exit(0);
    }

    // 如果没有提供 URL，显示帮助信息
    let mut url = matches.get_one::<String>("url")
        .ok_or_else(|| crate::error::Error::Custom("URL is required".to_string()))?
        .clone();
    
    // 如果 URL 没有协议前缀，添加 https://
    if !url.starts_with("http://") && !url.starts_with("https://") {
        url = format!("https://{}", url);
    }

    // 加载配置文件
    let config_file = if let Some(config_path) = matches.get_one::<String>("config") {
        ConfigFile::load_from_file(config_path)?
    } else {
        ConfigFile::load_default()
    };

    // 从命令行参数覆盖配置文件的设置
    let output_dir = matches.get_one::<String>("output")
        .map(|s| s.clone())
        .unwrap_or(config_file.general.output_dir.clone());
    
    let max_pages: u32 = matches.get_one::<String>("max-pages")
        .map(|s| s.parse())
        .unwrap_or(Ok(config_file.general.max_pages))?;
    
    let batch_size: usize = matches.get_one::<String>("batch-size")
        .map(|s| s.parse())
        .unwrap_or(Ok(config_file.general.batch_size))?;
    
    let wait_time: u64 = matches.get_one::<String>("wait-time")
        .map(|s| s.parse())
        .unwrap_or(Ok(config_file.general.wait_time))?;

    // 翻译配置：命令行参数优先
    let translate_enabled = if matches.get_flag("translate") {
        true
    } else {
        config_file.translation.enabled
    };

    let source_lang = matches.get_one::<String>("source-lang")
        .map(|s| s.clone())
        .unwrap_or(config_file.translation.source_lang.clone());
    
    let target_lang = matches.get_one::<String>("target-lang")
        .map(|s| s.clone())
        .unwrap_or(config_file.translation.target_lang.clone());
    
    let deeplx_url = matches.get_one::<String>("deeplx-url")
        .map(|s| s.clone())
        .unwrap_or(config_file.translation.deeplx_api_url.clone());

    let translation_config = TranslationConfig {
        enabled: translate_enabled,
        source_lang,
        target_lang,
        deeplx_api_url: deeplx_url,
        max_requests_per_second: config_file.translation.max_requests_per_second,
        max_text_length: config_file.translation.max_text_length,
        max_paragraphs_per_request: config_file.translation.max_paragraphs_per_request,
    };

    let config = Config::from_config_file(url, &config_file)
        .with_translation(translation_config);
    
    // 用命令行参数覆盖
    let mut config = config;
    config.output_dir = output_dir;
    config.max_pages = max_pages;
    config.batch_size = batch_size;
    config.wait_time = wait_time;
    
    config.validate()?;
    
    Ok(config)
}