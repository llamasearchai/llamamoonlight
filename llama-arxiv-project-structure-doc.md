# Llama-Arxiv Project Structure Documentation

## Overview

Llama-Arxiv is a robust command-line tool written in Rust for downloading, processing, and organizing papers from the arXiv repository. This document explains the project's structure and how different components work together.

## Directory Structure

```
llama-arxiv/
├── Cargo.toml                  # Project configuration and dependencies
├── LICENSE                     # MIT License
├── README.md                   # Project documentation
├── .github/
│   └── workflows/
│       └── rust.yml            # GitHub Actions CI configuration
├── src/
│   ├── main.rs                 # Main application entry point
│   ├── error.rs                # Error handling definitions
│   ├── utils.rs                # Utility functions
│   ├── modules/
│   │   ├── mod.rs              # Module aggregation
│   │   ├── cli.rs              # Command-line interface
│   │   ├── arxiv.rs            # arXiv API integration
│   │   ├── download.rs         # PDF downloading functionality
│   │   ├── parser.rs           # PDF parsing functions
│   │   ├── metadata.rs         # Paper metadata structures
│   │   └── config.rs           # Configuration management
│   └── tests/
│       └── integration_test.rs # Integration tests
```

## Core Components

### 1. Command-Line Interface (`modules/cli.rs`)

This module defines the command-line arguments and options using `clap`. Key features include:
- Accepting multiple arXiv IDs or URLs
- Configurable output directory
- Output format selection (text, HTML, markdown)
- Citation extraction option
- Verbose output flag

### 2. arXiv API Integration (`modules/arxiv.rs`)

Handles communication with the arXiv API to fetch paper metadata:
- Sends requests to the arXiv API
- Parses XML responses into Rust structures
- Extracts essential metadata (title, authors, abstract, etc.)
- Identifies PDF download links

### 3. Download Manager (`modules/download.rs`)

Manages the downloading of PDF files:
- Configurable HTTP client with timeouts
- Progress bar for download tracking
- Proper error handling for network issues
- File existence checks to avoid duplicate downloads

### 4. PDF Parser (`modules/parser.rs`)

Processes downloaded PDFs:
- Extracts text from PDF files (with PDFium when available)
- Converts PDF content to different formats (text, HTML, markdown)
- Attempts to extract BibTeX citations from PDF content
- Falls back gracefully when needed features aren't available

### 5. Metadata Handling (`modules/metadata.rs`)

Defines structures for paper metadata:
- Storage for paper details (title, authors, abstract, etc.)
- Proper serialization/deserialization with serde
- Display formatting for user output
- Utility methods for working with metadata

### 6. Configuration Management (`modules/config.rs`)

Manages user preferences through TOML configuration files:
- Default settings for timeout, retries, etc.
- Download directory configuration
- PDF processing options
- Loading/saving configuration from/to files

### 7. Error Handling (`error.rs`)

Centralized error handling with thiserror:
- Custom error types for different failure scenarios
- Proper error propagation throughout the application
- User-friendly error messages
- Conversion from library errors to application errors

### 8. Utilities (`utils.rs`)

Common utility functions used throughout the application:
- File and directory operations
- String sanitization
- URL validation
- Filename generation from metadata

## Execution Flow

1. The user invokes the application with arXiv IDs or URLs
2. Command-line arguments are parsed (`cli.rs`)
3. Configuration is loaded if specified (`config.rs`)
4. For each target:
   - Metadata is fetched from arXiv API (`arxiv.rs`)
   - The PDF is downloaded (`download.rs`)
   - If requested, the PDF is processed (`parser.rs`)
   - Results are saved to the specified directory
5. Summary information is displayed to the user

## Testing Strategy

- Unit tests for individual components
- Integration tests that verify the complete workflow
- Temporary directories for test outputs
- Mocking of external services where appropriate

## Extending the Project

To add new features to Llama-Arxiv:

1. For new commands: Modify `modules/cli.rs` to add options
2. For new output formats: Add conversion functions to `modules/parser.rs`
3. For additional metadata extraction: Enhance `modules/arxiv.rs`
4. For new configuration options: Update `modules/config.rs`

## Build and Deployment

The project uses GitHub Actions for CI/CD:
- Automated testing on push
- Release creation on tags
- Binary artifacts for easy distribution

## Future Improvements

Potential enhancements for future versions:
- Database integration for paper tracking
- Full-text search across downloaded papers
- Integration with reference management software
- PDF annotation capabilities
- Browser extension for one-click downloads from arXiv
