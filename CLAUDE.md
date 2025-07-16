# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

This is a Rust-based CLI tool that crawls websites and converts all pages to Markdown files with optional translation support. It uses the `spider` crate for web crawling, the `r.jina.ai` API to convert HTML content to high-quality Markdown format, and supports DeepLX API for translation services.

## Development Commands

### Build and Run
```bash
# Build the project
cargo build --release

# Run in development mode
cargo run -- <URL> [OPTIONS]

# Run with specific options
cargo run -- https://example.com -o ./output -m 20

# Run with translation enabled
cargo run -- https://example.com -t --source-lang en --target-lang zh

# Run with custom DeepLX API
cargo run -- https://example.com -t --deeplx-url http://localhost:1188/translate

# Generate example configuration file
cargo run -- --generate-config config.toml

# Run with custom configuration file
cargo run -- https://example.com -c config.toml

# Build optimized release version
cargo build --release
./target/release/markdown-downloader <URL> [OPTIONS]
```

### Development Tools
```bash
# Run tests
cargo test

# Format code
cargo fmt

# Lint code
cargo clippy

# Enable debug logging
RUST_LOG=debug cargo run -- <URL>
```

## Architecture Overview

### Core Components

1. **Main Entry Point** (`src/main.rs`):
   - CLI argument parsing using `clap`
   - Async main function with tokio runtime
   - Concurrent batch processing with configurable concurrency limits
   - Progress tracking and status reporting
   - Integration with translation and file management services
   - Error handling with retry mechanisms

2. **CLI Interface** (`src/cli.rs`):
   - Command-line argument parsing and validation
   - Configuration file integration with command-line override
   - URL normalization (auto-adds https:// prefix)
   - Config file generation functionality

3. **Configuration System**:
   - **Config** (`src/config.rs`): Runtime configuration management
   - **ConfigFile** (`src/config_file.rs`): TOML configuration file handling
   - **Types** (`src/types.rs`): Configuration data structures
   - Automatic config file discovery in multiple locations
   - Command-line arguments take precedence over config files

4. **Web Crawling** (`src/crawler.rs`):
   - Uses `spider` crate to discover all website links
   - Configured with robots.txt compliance and rate limiting
   - Built-in 1-second delay between requests
   - Configurable maximum pages limit

5. **Markdown Conversion** (`src/converter.rs`):
   - `convert_url_to_markdown()` function calls `r.jina.ai` API
   - Converts HTML pages to Markdown format
   - Configurable timeout for HTTP requests
   - Proper error handling for HTTP failures

6. **Translation Service** (`src/translator.rs`):
   - DeepLX API integration for content translation
   - Intelligent text chunking for long content (handles paragraphs and sentences)
   - Advanced rate limiting with semaphore-based throttling
   - Retry mechanisms with exponential backoff
   - Support for multiple API formats (DeepLX and dptrans)
   - Automatic language detection and target language configuration

7. **File Management**:
   - **Folder Manager** (`src/folder_manager.rs`): Organizes files into structured directories
   - **Links Manager** (`src/links_manager.rs`): Tracks processed URLs and prevents duplicates
   - **File Manager** (`src/file_manager.rs`): Basic file operations
   - **Progress Tracker** (`src/progress.rs`): Progress reporting and logging
   - Format: `domain_YYYYMMDD_path_timestamp.md`
   - Handles special characters and path sanitization
   - Bilingual content creation with side-by-side comparison

8. **Folder Structure**:
   - **Without Translation**: Single folder with domain name
   - **With Translation**: Three subfolders per domain-language pair:
     - `original/`: Original markdown files
     - `translated/`: Translated markdown files  
     - `bilingual/`: Side-by-side bilingual content

### Key Configuration

- **Rate Limiting**: 1000ms delay between spider requests + configurable request_delay between downloads
- **Translation Rate Limiting**: Configurable API requests per second (default: 0.5)
- **Batch Processing**: Configurable batch size with wait periods between batches
- **Concurrency**: Limited to min(batch_size, 3) to prevent API overload
- **Timeout**: Configurable HTTP timeout for API calls (default: 30 seconds)
- **User Agent**: "Mozilla/5.0 (compatible; MarkdownDownloader/1.0)"
- **Text Chunking**: Intelligent paragraph-based chunking with configurable max length
- **Retry Logic**: Exponential backoff with configurable retry attempts

### Processing Flow

1. **URL Discovery**: Spider crawls the website and discovers all links
2. **Link Filtering**: Links manager filters out already processed URLs
3. **Batch Processing**: URLs are processed in configurable batches
4. **Markdown Conversion**: Each URL is converted to Markdown via r.jina.ai
5. **Translation** (if enabled): Content is translated using DeepLX API
6. **File Storage**: Content is saved in organized folder structure
7. **Progress Tracking**: Status is tracked and logged throughout the process

### Command Line Interface

```bash
markdown-downloader <URL> [OPTIONS]

Options:
  -o, --output <DIR>        Output directory (default: "./downloads")
  -m, --max-pages <NUM>     Maximum pages to crawl (default: 50)
  -b, --batch-size <NUM>    Pages to process before waiting (default: 10)
  -w, --wait-time <SECS>    Wait time between batches (default: 60)
  -t, --translate           Enable translation
      --source-lang <LANG>  Source language for translation (default: "auto")
      --target-lang <LANG>  Target language for translation (default: "zh")
      --deeplx-url <URL>    DeepLX API URL (default: "http://localhost:1188/translate")
  -c, --config <FILE>       Path to configuration file
      --generate-config <FILE>  Generate example configuration file
```

## External Dependencies

- **spider**: Web crawling and link discovery with robots.txt compliance
- **reqwest**: HTTP client for r.jina.ai and DeepLX API calls with timeout and connection pooling
- **tokio**: Async runtime for concurrent processing
- **clap**: Command-line argument parsing with subcommands support
- **chrono**: Date/time formatting for filenames and timestamps
- **url**: URL parsing and manipulation
- **serde/serde_json**: JSON serialization for API requests and responses
- **toml**: Configuration file parsing
- **dirs**: User directory discovery for config file locations
- **futures**: Stream processing for concurrent URL handling

## Error Handling

The application includes comprehensive error handling for:
- Network connectivity issues
- HTTP errors from r.jina.ai and DeepLX APIs
- Translation service failures with retry mechanisms
- File system operations
- URL parsing failures
- Rate limiting and timeout scenarios
- Malformed API responses

## Translation Features

### Supported Languages
- **Source Language**: Auto-detection or manual specification
- **Target Language**: Configurable (default: Chinese)
- **API Support**: DeepLX and compatible translation services

### File Organization
When translation is enabled, files are organized as follows:
```
output_directory/
└── domain.com_en-zh/
    ├── original/        # Original markdown files
    ├── translated/      # Translated markdown files
    └── bilingual/       # Side-by-side comparison
```

### Progress Tracking
- **Links File**: `domain_com_links.txt` tracks processing status
- **Status Indicators**: ✅ for success, ❌ for failures
- **Resume Support**: Automatically skips already processed URLs

## Configuration Files

The application supports configuration files to set default values and avoid repetitive command-line arguments.

### Configuration File Locations
The tool automatically searches for configuration files in this order:
1. `markdown-downloader.toml` (current directory)
2. `config.toml` (current directory)
3. `.markdown-downloader.toml` (current directory)
4. `~/.config/markdown-downloader/config.toml` (user config directory)

### Configuration File Format
```toml
[general]
output_dir = "./downloads"
max_pages = 50
batch_size = 10
wait_time = 60
request_delay = 500
timeout = 30
user_agent = "Mozilla/5.0 (compatible; MarkdownDownloader/1.0)"

[translation]
enabled = false
source_lang = "auto"
target_lang = "zh"
deeplx_api_url = "http://localhost:1188/translate"
max_requests_per_second = 0.5
max_text_length = 3000
max_paragraphs_per_request = 10
```

### Command-Line Override
Command-line arguments always take precedence over configuration file settings, allowing you to override specific options as needed.

## Key Implementation Details

### Rate Limiting and Throttling
- The translation service uses a semaphore-based rate limiter
- Request delays are enforced between API calls to prevent 429 errors
- Batch processing includes wait periods between batches

### Text Processing
- Long content is intelligently split at paragraph boundaries
- Sentence-level splitting for very long paragraphs
- Maintains markdown formatting during translation

### Error Recovery
- Failed URLs are logged with error details in links file
- Translation failures don't stop the entire process
- Network errors trigger retry with exponential backoff

### File Naming Convention
Files are named using the pattern: `domain_YYYYMMDD_path_timestamp.md`
- Domain: Website hostname with dots replaced by underscores
- Date: Current date in YYYYMMDD format
- Path: URL path with special characters sanitized
- Timestamp: Unix timestamp for uniqueness

### Resume Capability
The application supports resuming interrupted processes:
- Already processed URLs are tracked in `domain_com_links.txt`
- Status indicators: ✅ for success, ❌ for failures
- Automatic filtering of completed URLs on restart