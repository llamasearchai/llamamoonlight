# Llama ArXiv 📚🦙

A command-line tool for fetching, downloading, and processing arXiv papers with ease.

## Features

- 📥 **Download papers** - Easily download PDFs from arXiv by ID or URL
- 🔍 **Extract metadata** - Get detailed paper information from the arXiv API
- 📄 **Text extraction** - Convert PDF papers to text, Markdown, or HTML
- 📑 **Section detection** - Automatically identify and extract paper sections
- 📚 **Citation export** - Generate BibTeX citations for papers
- 🔄 **Batch processing** - Process multiple papers in one command

## Installation

### Using Cargo

```bash
cargo install llama-arxiv
```

### From Source

```bash
git clone https://github.com/yourusername/llama-arxiv.git
cd llama-arxiv
cargo build --release
```

The binary will be available at `target/release/llama-arxiv`.

## Usage

### Basic Usage

```bash
# Download a paper by arXiv ID
llama-arxiv 2103.13630

# Download a paper from URL
llama-arxiv https://arxiv.org/abs/2103.13630

# Download multiple papers
llama-arxiv 2103.13630 1706.03762 2005.14165
```

### Output Options

```bash
# Download and extract text (default)
llama-arxiv --format text 2103.13630

# Convert to markdown
llama-arxiv --format markdown 2103.13630

# Convert to HTML
llama-arxiv --format html 2103.13630

# Specify output directory
llama-arxiv --output-dir ~/papers 2103.13630
```

### Other Options

```bash
# Only fetch metadata (don't download PDF)
llama-arxiv --metadata-only 2103.13630

# Only download PDF (don't extract text)
llama-arxiv --download-only 2103.13630

# Force re-download of existing files
llama-arxiv --force 2103.13630

# Generate BibTeX citation file
llama-arxiv --citations 2103.13630

# Increase verbosity
llama-arxiv --verbose 2103.13630

# Suppress all output except errors
llama-arxiv --quiet 2103.13630
```

## Configuration

Llama ArXiv uses a configuration file to customize its behavior. By default, it looks for a config file at `~/.config/llama-arxiv/config.toml`.

You can specify a different configuration file using the `--config` option:

```bash
llama-arxiv --config my-config.toml 2103.13630
```

If no configuration file exists, a default one will be created for you.

### Example Configuration

```toml
[api]
base_url = "http://export.arxiv.org/api/query"
user_agent = "LlamaArxiv/0.1.0"
timeout = 30
max_results = 100

[download]
download_dir = "~/Downloads/arxiv"
use_id_as_filename = false
timeout = 60

[pdf]
extract_sections = true
extract_references = true
```

## Examples

### Download a paper and convert to markdown

```bash
llama-arxiv --format markdown 2103.13630
```

### Download multiple papers and generate citations

```bash
llama-arxiv --citations 2103.13630 1706.03762 2005.14165
```

### Only fetch metadata for a paper

```bash
llama-arxiv --metadata-only 2103.13630
```

## License

This project is licensed under the MIT License - see the LICENSE file for details. 