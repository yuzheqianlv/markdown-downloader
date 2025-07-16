# Markdown Downloader

一个功能强大的 Rust 命令行工具，支持爬取网站并将所有页面转换为 Markdown 文件，具备智能翻译功能。使用 `spider` 进行网站爬取，通过 `r.jina.ai` API 将网页内容转换为高质量的 Markdown 格式，并可选择使用 DeepLX API 进行内容翻译。

## ✨ 功能特点

### 🕷️ 核心功能
- **智能爬取**: 使用 `spider` crate 自动发现网站所有链接
- **Markdown 转换**: 通过 `r.jina.ai` API 将网页内容转换为 Markdown
- **智能命名**: 文件名格式为 `域名_日期_路径.md`
- **遵守规则**: 自动遵守 robots.txt 规则

### 🌐 翻译功能
- **DeepLX 支持**: 集成 DeepLX API 进行内容翻译
- **智能分块**: 自动处理长文本，支持大型文档翻译
- **多语言支持**: 支持自动语言检测和多目标语言翻译
- **三种文件格式**: 原文、译文、双语对照

### 📁 文件管理
- **智能组织**: 按域名和语言自动创建文件夹结构
- **断点续传**: 支持任务中断后继续处理
- **状态跟踪**: 记录处理状态，防止重复处理

### ⚙️ 配置管理
- **配置文件**: 支持 TOML 格式配置文件
- **灵活配置**: 命令行参数和配置文件相结合
- **自动发现**: 多种配置文件路径自动搜索

## 🚀 安装与部署

### 从源码编译

```bash
git clone https://github.com/your-username/markdown-downloader.git
cd markdown-downloader
cargo build --release
```

### 系统安装（推荐）

编译完成后，可以将二进制文件安装到系统路径：

```bash
# 方式1：复制到 /usr/local/bin (需要 sudo)
sudo cp target/release/markdown-downloader /usr/local/bin/downloader

# 方式2：复制到用户 bin 目录
mkdir -p ~/.local/bin
cp target/release/markdown-downloader ~/.local/bin/downloader
```

### 环境变量配置

将以下内容添加到你的 shell 配置文件（`~/.bashrc`, `~/.zshrc`, `~/.profile` 等）：

```bash
# 添加用户 bin 目录到 PATH（如果使用方式2）
export PATH="$HOME/.local/bin:$PATH"

# 或者创建别名
alias downloader="$HOME/path/to/markdown-downloader/target/release/markdown-downloader"
```

重新加载配置：

```bash
source ~/.bashrc  # 或你使用的 shell 配置文件
```

### 验证安装

```bash
# 验证 downloader 命令可用
downloader --help

# 生成示例配置文件
downloader --generate-config config.toml
```

## 📖 使用方法

### 基本用法

```bash
# 下载网站的所有页面为 Markdown 文件
downloader https://example.com

# 指定输出目录
downloader https://docs.rust-lang.org/book/ -o ./rust-book

# 限制最大页面数
downloader https://docs.python.org/3/ -m 50
```

### 翻译功能

```bash
# 启用翻译功能
downloader https://example.com -t

# 指定源语言和目标语言
downloader https://example.com -t --source-lang en --target-lang zh

# 使用自定义 DeepLX API 地址
downloader https://example.com -t --deeplx-url http://your-server:1188/translate
```

### 配置文件使用

```bash
# 生成示例配置文件
downloader --generate-config config.toml

# 使用指定配置文件
downloader https://example.com -c config.toml

# 配置文件 + 命令行参数（命令行优先）
downloader https://example.com -c config.toml -t -m 100
```

### 组合使用示例

```bash
# 完整功能使用
downloader https://docs.example.com \\
  -o ./docs-backup \\
  -m 200 \\
  -t \\
  --source-lang en \\
  --target-lang zh \\
  --deeplx-url http://localhost:1188/translate
```

## 🔧 命令行参数

| 参数 | 短参数 | 描述 | 默认值 |
|------|--------|------|--------|
| `<URL>` | - | 目标网站 URL（必需） | - |
| `--output` | `-o` | 输出目录 | `./downloads` |
| `--max-pages` | `-m` | 最大爬取页面数 | `50` |
| `--batch-size` | `-b` | 批处理大小 | `3` |
| `--wait-time` | `-w` | 批次间等待时间（秒） | `90` |
| `--translate` | `-t` | 启用翻译功能 | `false` |
| `--source-lang` | - | 源语言 | `auto` |
| `--target-lang` | - | 目标语言 | `zh` |
| `--deeplx-url` | - | DeepLX API 地址 | `http://localhost:1188/translate` |
| `--config` | `-c` | 配置文件路径 | 自动搜索 |
| `--generate-config` | - | 生成示例配置文件 | - |

## 📁 输出文件结构

### 普通模式（无翻译）
```
downloads/
└── example.com/
    ├── example.com_20250715_index_1721234567.md
    ├── example.com_20250715_about_1721234568.md
    └── example.com_20250715_contact_1721234569.md
```

### 翻译模式
```
downloads/
└── example.com_en-zh/
    ├── original/          # 原文 Markdown 文件
    │   ├── example.com_20250715_index_1721234567.md
    │   └── example.com_20250715_about_1721234568.md
    ├── translated/        # 译文 Markdown 文件
    │   ├── example.com_20250715_index_1721234567.md
    │   └── example.com_20250715_about_1721234568.md
    └── bilingual/         # 双语对照文件
        ├── example.com_20250715_index_1721234567.md
        └── example.com_20250715_about_1721234568.md
```

### 处理状态文件
```
downloads/
├── example_com_links.txt  # 链接处理状态记录
└── example.com_en-zh/     # 翻译文件夹
```

## ⚙️ 配置文件

### 配置文件位置

工具会按以下顺序搜索配置文件：

1. `markdown-downloader.toml` (当前目录)
2. `config.toml` (当前目录)
3. `.markdown-downloader.toml` (当前目录)
4. `~/.config/markdown-downloader/config.toml` (用户配置目录)

### 配置文件格式

```toml
[general]
output_dir = "./downloads"
max_pages = 50
batch_size = 3                   # 批处理大小，建议值：1-10
wait_time = 90                   # 批次间等待时间（秒），建议值：30-120
request_delay = 1000             # 请求延迟（毫秒），建议值：500-2000
timeout = 30                     # 请求超时时间（秒）
user_agent = "Mozilla/5.0 (compatible; MarkdownDownloader/1.0)"

[translation]
enabled = false
source_lang = "auto"             # 源语言：auto/en/zh/ja/ko等
target_lang = "zh"               # 目标语言：zh/en/ja/ko等
deeplx_api_url = "http://localhost:1188/translate"
max_requests_per_second = 0.5    # 翻译请求频率（次/秒），建议值：0.2-2.0
max_text_length = 2000           # 单次翻译文本最大长度，建议值：1000-5000
max_paragraphs_per_request = 5   # 单次翻译最大段落数，建议值：3-15
```

### 生成配置文件

```bash
# 生成示例配置文件
downloader --generate-config my-config.toml

# 编辑配置文件
nano my-config.toml

# 使用配置文件
downloader https://example.com -c my-config.toml
```

## 🌐 DeepLX 配置

### 本地部署 DeepLX

```bash
# 使用 Docker 运行 DeepLX
docker run -d -p 1188:1188 ghcr.io/owo-network/deeplx:latest

# 测试 DeepLX 服务
curl -X POST http://localhost:1188/translate \\
  -H "Content-Type: application/json" \\
  -d '{"text":"Hello World","source_lang":"auto","target_lang":"zh"}'
```

### 远程 DeepLX 服务

```bash
# 使用远程 DeepLX 服务
downloader https://example.com -t --deeplx-url http://your-server:1188/translate
```

## 💡 使用示例

### 技术文档翻译

```bash
# Rust 官方文档翻译
downloader https://doc.rust-lang.org/book/ \\
  -o ./rust-book-zh \\
  -m 50 \\
  -t \\
  --source-lang en \\
  --target-lang zh

# Python 文档翻译
downloader https://docs.python.org/3/ \\
  -o ./python-docs-zh \\
  -m 100 \\
  -t
```

### 博客备份与翻译

```bash
# 个人博客备份
downloader https://blog.example.com -o ./blog-backup -m 200

# 企业文档翻译
downloader https://docs.company.com \\
  -o ./company-docs-zh \\
  -m 500 \\
  -t \\
  --target-lang zh
```

### 配置文件使用

```bash
# 创建专用配置
cat > docs-config.toml << EOF
[general]
output_dir = "./translated-docs"
max_pages = 100
batch_size = 5
wait_time = 120

[translation]
enabled = true
source_lang = "en"
target_lang = "zh"
deeplx_api_url = "http://localhost:1188/translate"
max_requests_per_second = 1.0
EOF

# 使用配置文件
downloader https://docs.example.com -c docs-config.toml
```

## 🛠️ 项目架构

### 核心模块

- **`src/main.rs`** - 应用程序入口点
- **`src/cli.rs`** - 命令行参数解析
- **`src/config.rs`** - 运行时配置管理
- **`src/config_file.rs`** - 配置文件处理
- **`src/crawler.rs`** - 网站爬取功能
- **`src/converter.rs`** - Markdown 转换服务
- **`src/translator.rs`** - 翻译服务
- **`src/folder_manager.rs`** - 文件夹结构管理
- **`src/links_manager.rs`** - 链接状态管理
- **`src/file_manager.rs`** - 文件操作
- **`src/progress.rs`** - 进度追踪
- **`src/types.rs`** - 类型定义
- **`src/error.rs`** - 错误处理

### 作为库使用

```rust
use markdown_downloader::{
    Config, Crawler, MarkdownConverter, TranslationService, 
    FolderManager, LinksManager
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::new(
        "https://example.com".to_string(),
        "./output".to_string(),
        10, 5, 30,
    );
    
    // 初始化服务
    let mut crawler = Crawler::new(&config)?;
    let converter = MarkdownConverter::new(config.clone());
    let translator = TranslationService::new(config.translation.clone());
    let folder_manager = FolderManager::new(
        std::path::PathBuf::from(&config.output_dir),
        config.translation.clone()
    );
    
    // 处理流程
    let urls = crawler.crawl().await?;
    for url in urls {
        let content = converter.convert_url_to_markdown(&url).await?;
        let translated = translator.translate(&content).await?;
        folder_manager.save_content(&url, &content, Some(&translated))?;
    }
    
    Ok(())
}
```

## 🔍 故障排除

### 常见问题

**1. 编译错误**
```bash
# 确保 Rust 版本足够新
rustc --version
rustup update

# 清理缓存重新编译
cargo clean
cargo build --release
```

**2. 命令未找到**
```bash
# 检查 PATH 设置
echo $PATH

# 检查文件是否存在
ls -la ~/.local/bin/downloader

# 重新加载 shell 配置
source ~/.bashrc
```

**3. 网络超时**
```bash
# 检查网络连接
ping r.jina.ai
curl -I https://r.jina.ai

# 使用代理（如果需要）
export HTTP_PROXY=http://proxy.example.com:8080
export HTTPS_PROXY=http://proxy.example.com:8080
```

**4. 翻译服务错误**
```bash
# 检查 DeepLX 服务状态
curl http://localhost:1188/translate

# 启动 DeepLX 服务
docker run -d -p 1188:1188 ghcr.io/owo-network/deeplx:latest

# 检查服务日志
docker logs <container_id>
```

**5. 文件权限错误**
```bash
# 确保输出目录有写权限
chmod 755 ./downloads
mkdir -p ./downloads
```

### 调试模式

```bash
# 启用详细日志
RUST_LOG=debug downloader https://example.com

# 查看详细错误信息
RUST_BACKTRACE=1 downloader https://example.com
```

## 📊 性能优化

### 配置参数详解

#### 基本配置参数

- **`batch_size`** (批处理大小): 控制并发处理的链接数量
  - 推荐值：1-10
  - 较小值：减少服务器压力，避免429错误
  - 较大值：提高处理速度，但可能触发限制

- **`wait_time`** (批次间等待时间): 每批处理完成后的等待时间（秒）
  - 推荐值：30-120
  - 用于避免API频率限制和服务器过载

- **`request_delay`** (请求延迟): 单个请求间的延迟（毫秒）
  - 推荐值：500-2000
  - 影响下载和翻译请求的间隔

#### 翻译参数

- **`max_requests_per_second`** (翻译请求频率): 每秒最大翻译请求数
  - 推荐值：0.2-2.0
  - 过高可能导致429错误，过低影响效率

- **`max_text_length`** (单次翻译长度): 单次翻译的文本最大长度
  - 推荐值：1000-5000
  - 过长可能导致翻译质量下降或API错误

- **`max_paragraphs_per_request`** (单次翻译段落数): 单次翻译的最大段落数
  - 推荐值：3-15
  - 控制翻译粒度和效率

### 预设配置方案

#### 🚀 高性能配置 (config.performance.toml)
适用于本地网络环境或高性能API服务：

```toml
[general]
batch_size = 5
wait_time = 60
request_delay = 500

[translation]
max_requests_per_second = 1.0
max_text_length = 3000
max_paragraphs_per_request = 8
```

#### 🛡️ 稳定配置 (config.stable.toml)
适用于公网环境或API限制较严格的场景：

```toml
[general]
batch_size = 1
wait_time = 60
request_delay = 1000

[translation]
max_requests_per_second = 0.5
max_text_length = 2000
max_paragraphs_per_request = 5
```

#### ⚡ 默认配置 (config.toml)
平衡性能和稳定性：

```toml
[general]
batch_size = 3
wait_time = 90
request_delay = 1000

[translation]
max_requests_per_second = 0.5
max_text_length = 2000
max_paragraphs_per_request = 5
```

### 性能调优建议

1. **遇到429错误时**：
   - 降低 `max_requests_per_second` 到 0.2-0.5
   - 增加 `wait_time` 到 120-180秒
   - 减少 `batch_size` 到 1-3

2. **网络环境良好时**：
   - 可适当提高 `max_requests_per_second` 到 1.0-2.0
   - 减少 `wait_time` 到 30-60秒
   - 增加 `batch_size` 到 5-10

3. **处理大量短文本时**：
   - 减少 `max_text_length` 到 1000-2000
   - 增加 `max_paragraphs_per_request` 到 10-20

4. **处理长文档时**：
   - 增加 `max_text_length` 到 3000-5000
   - 减少 `max_paragraphs_per_request` 到 3-8

### 使用不同配置文件

```bash
# 使用高性能配置
downloader https://example.com -c config.performance.toml

# 使用稳定配置
downloader https://example.com -c config.stable.toml

# 使用默认配置
downloader https://example.com -c config.toml
```

### 系统要求

- **CPU**: 多核心处理器推荐
- **内存**: 至少 512MB 可用内存
- **网络**: 稳定的网络连接
- **磁盘**: 足够的存储空间用于输出文件

## 🤝 贡献指南

### 开发环境设置

```bash
git clone https://github.com/your-username/markdown-downloader.git
cd markdown-downloader

# 安装开发依赖
cargo install cargo-watch
cargo install cargo-audit

# 运行测试
cargo test

# 代码格式化
cargo fmt

# 代码检查
cargo clippy

# 安全审计
cargo audit
```

### 开发工作流

```bash
# 开发时自动重新编译
cargo watch -x "run -- https://example.com -m 5"

# 运行所有检查
cargo test && cargo fmt && cargo clippy
```

## 📄 许可证

MIT License - 详见 [LICENSE](LICENSE) 文件

## 🙏 致谢

- [spider](https://crates.io/crates/spider) - 强大的网页爬虫库
- [r.jina.ai](https://r.jina.ai) - 优秀的网页到 Markdown 转换服务
- [DeepLX](https://github.com/OwO-Network/DeepLX) - 免费的 DeepL 翻译 API
- [clap](https://crates.io/crates/clap) - 命令行参数解析
- [reqwest](https://crates.io/crates/reqwest) - HTTP 客户端库
- [tokio](https://crates.io/crates/tokio) - 异步运行时
- [serde](https://crates.io/crates/serde) - 序列化框架

---

## 📞 支持

如果您在使用过程中遇到问题，请：

1. 查看本文档的故障排除部分
2. 搜索已有的 [Issues](https://github.com/your-username/markdown-downloader/issues)
3. 创建新的 Issue 描述问题

**免责声明**: 请遵守目标网站的 robots.txt 和使用条款，合理使用本工具。翻译功能依赖第三方服务，请确保遵守相关服务条款。