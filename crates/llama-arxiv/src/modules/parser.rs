use std::path::{Path, PathBuf};
use std::fs;
use std::io::{Read, Write};
use thiserror::Error;
use lopdf::{Document, Object, Dictionary};
use log::{debug, info, warn, error};
use regex::Regex;
use lazy_static::lazy_static;

use crate::modules::config::PdfConfig;
use crate::modules::metadata::PaperMetadata;

/// Error types for PDF parsing operations
#[derive(Error, Debug)]
pub enum ParserError {
    #[error("File system error: {0}")]
    FileSystem(#[from] std::io::Error),
    
    #[error("PDF parsing error: {0}")]
    PdfParse(#[from] lopdf::Error),
    
    #[error("Text extraction error: {0}")]
    TextExtraction(String),
    
    #[error("Processing error: {0}")]
    Processing(String),
}

/// Result type for parser operations
pub type ParserResult<T> = Result<T, ParserError>;

/// Structure representing a parsed PDF document
#[derive(Debug)]
pub struct ParsedPdf {
    /// Full text content of the PDF
    pub text: String,
    
    /// Extracted sections of the PDF
    pub sections: Vec<PdfSection>,
    
    /// References extracted from the PDF
    pub references: Vec<String>,
    
    /// Path to the original PDF file
    pub source_path: PathBuf,
    
    /// Associated metadata if available
    pub metadata: Option<PaperMetadata>,
}

/// Structure representing a section in a PDF
#[derive(Debug, Clone)]
pub struct PdfSection {
    /// Section heading
    pub heading: String,
    
    /// Section content
    pub content: String,
    
    /// Section level (1 = top level)
    pub level: u8,
}

impl ParsedPdf {
    /// Create a new parsed PDF with text content
    pub fn new(text: String, source_path: PathBuf) -> Self {
        Self {
            text,
            sections: Vec::new(),
            references: Vec::new(),
            source_path,
            metadata: None,
        }
    }
    
    /// Get the abstract section if it exists
    pub fn abstract_text(&self) -> Option<&str> {
        self.sections
            .iter()
            .find(|s| s.heading.to_lowercase().contains("abstract"))
            .map(|s| s.content.as_str())
    }
    
    /// Get the introduction section if it exists
    pub fn introduction(&self) -> Option<&str> {
        self.sections
            .iter()
            .find(|s| {
                s.heading.to_lowercase().contains("introduction") ||
                s.heading.to_lowercase().contains("1. introduction")
            })
            .map(|s| s.content.as_str())
    }
    
    /// Get the conclusion section if it exists
    pub fn conclusion(&self) -> Option<&str> {
        self.sections
            .iter()
            .find(|s| {
                s.heading.to_lowercase().contains("conclusion") ||
                s.heading.to_lowercase().contains("summary") ||
                s.heading.to_lowercase().contains("discussion")
            })
            .map(|s| s.content.as_str())
    }
    
    /// Save the extracted text to a file
    pub fn save_text(&self, output_path: &Path) -> ParserResult<()> {
        // Create parent directories if they don't exist
        if let Some(parent) = output_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        
        fs::write(output_path, &self.text)?;
        Ok(())
    }
    
    /// Save in Markdown format
    pub fn save_markdown(&self, output_path: &Path) -> ParserResult<()> {
        // Create parent directories if they don't exist
        if let Some(parent) = output_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        
        let mut markdown = String::new();
        
        // Add metadata if available
        if let Some(metadata) = &self.metadata {
            markdown.push_str(&format!("# {}\n\n", metadata.title));
            markdown.push_str(&format!("*Authors:* {}\n\n", metadata.authors.join(", ")));
            markdown.push_str(&format!("*ID:* {} (v{})\n\n", metadata.id, metadata.version));
            
            if !metadata.categories.is_empty() {
                markdown.push_str(&format!("*Categories:* {}\n\n", metadata.categories.join(", ")));
            }
            
            if let Some(ref doi) = metadata.doi {
                markdown.push_str(&format!("*DOI:* {}\n\n", doi));
            }
            
            if let Some(ref journal) = metadata.journal_ref {
                markdown.push_str(&format!("*Journal:* {}\n\n", journal));
            }
        }
        
        // Add sections
        for section in &self.sections {
            // Calculate header level (minimum h1, maximum h3)
            let level = std::cmp::min(section.level, 3);
            let heading_marks = "#".repeat(level as usize);
            
            markdown.push_str(&format!("{} {}\n\n", heading_marks, section.heading));
            markdown.push_str(&format!("{}\n\n", section.content));
        }
        
        // Add references if available
        if !self.references.is_empty() {
            markdown.push_str("## References\n\n");
            
            for (i, reference) in self.references.iter().enumerate() {
                markdown.push_str(&format!("{}. {}\n\n", i + 1, reference));
            }
        }
        
        fs::write(output_path, markdown)?;
        Ok(())
    }
    
    /// Save in HTML format
    pub fn save_html(&self, output_path: &Path) -> ParserResult<()> {
        // Create parent directories if they don't exist
        if let Some(parent) = output_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }
        
        let mut html = String::from("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
        html.push_str("<meta charset=\"UTF-8\">\n");
        html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
        
        // Add title if available
        if let Some(metadata) = &self.metadata {
            html.push_str(&format!("<title>{}</title>\n", metadata.title));
        } else {
            html.push_str("<title>Parsed PDF</title>\n");
        }
        
        // Add basic CSS
        html.push_str("<style>\n");
        html.push_str("body { font-family: Arial, sans-serif; line-height: 1.6; max-width: 800px; margin: 0 auto; padding: 20px; }\n");
        html.push_str("h1, h2, h3 { color: #333; }\n");
        html.push_str(".metadata { background-color: #f5f5f5; padding: 15px; border-radius: 5px; margin-bottom: 20px; }\n");
        html.push_str(".section { margin-bottom: 20px; }\n");
        html.push_str(".references { background-color: #f9f9f9; padding: 15px; border-radius: 5px; }\n");
        html.push_str("</style>\n");
        html.push_str("</head>\n<body>\n");
        
        // Add metadata if available
        if let Some(metadata) = &self.metadata {
            html.push_str("<div class=\"metadata\">\n");
            html.push_str(&format!("<h1>{}</h1>\n", metadata.title));
            html.push_str(&format!("<p><strong>Authors:</strong> {}</p>\n", metadata.authors.join(", ")));
            html.push_str(&format!("<p><strong>ID:</strong> {} (v{})</p>\n", metadata.id, metadata.version));
            
            if !metadata.categories.is_empty() {
                html.push_str(&format!("<p><strong>Categories:</strong> {}</p>\n", metadata.categories.join(", ")));
            }
            
            if let Some(ref doi) = metadata.doi {
                html.push_str(&format!("<p><strong>DOI:</strong> {}</p>\n", doi));
            }
            
            if let Some(ref journal) = metadata.journal_ref {
                html.push_str(&format!("<p><strong>Journal:</strong> {}</p>\n", journal));
            }
            
            html.push_str("</div>\n");
        }
        
        // Add sections
        for section in &self.sections {
            html.push_str("<div class=\"section\">\n");
            
            // Calculate header level (minimum h1, maximum h3)
            let level = std::cmp::min(section.level + 1, 4);
            html.push_str(&format!("<h{}>{}</h{}>\n", level, section.heading, level));
            
            // Split paragraphs and add them
            for paragraph in section.content.split("\n\n") {
                if !paragraph.trim().is_empty() {
                    html.push_str(&format!("<p>{}</p>\n", paragraph.trim()));
                }
            }
            
            html.push_str("</div>\n");
        }
        
        // Add references if available
        if !self.references.is_empty() {
            html.push_str("<div class=\"references\">\n");
            html.push_str("<h2>References</h2>\n");
            html.push_str("<ol>\n");
            
            for reference in &self.references {
                html.push_str(&format!("<li>{}</li>\n", reference));
            }
            
            html.push_str("</ol>\n");
            html.push_str("</div>\n");
        }
        
        html.push_str("</body>\n</html>");
        
        fs::write(output_path, html)?;
        Ok(())
    }
}

/// PDF parser for scientific papers
pub struct PdfParser {
    /// Parser configuration
    config: PdfConfig,
}

impl PdfParser {
    /// Create a new PDF parser
    pub fn new(config: PdfConfig) -> Self {
        Self { config }
    }
    
    /// Parse a PDF file and extract text content
    pub fn parse_pdf(&self, pdf_path: &Path) -> ParserResult<ParsedPdf> {
        debug!("Parsing PDF: {}", pdf_path.display());
        
        // Check if the file exists and is accessible
        if !pdf_path.exists() {
            return Err(ParserError::FileSystem(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("File not found: {}", pdf_path.display()),
            )));
        }
        
        // Attempt to parse the PDF document
        let doc = Document::load(pdf_path)?;
        
        // Extract text from the document
        let text = self.extract_text(&doc)?;
        
        // Create the parsed PDF object
        let mut parsed = ParsedPdf::new(text, pdf_path.to_path_buf());
        
        // Extract sections based on the configuration
        if self.config.extract_sections {
            parsed.sections = self.extract_sections(&parsed.text);
        }
        
        // Extract references based on the configuration
        if self.config.extract_references {
            parsed.references = self.extract_references(&parsed.text);
        }
        
        Ok(parsed)
    }
    
    /// Extract text content from a PDF document
    fn extract_text(&self, doc: &Document) -> ParserResult<String> {
        let mut buffer = String::new();
        
        // Extract text from all pages
        for (_, page_id) in doc.get_pages() {
            if let Ok(page) = doc.get_dictionary(page_id) {
                match page.get(b"Contents") {
                    Ok(content_ids) => {
                        // Get content from a single object or an array of objects
                        match content_ids {
                            // Single content object
                            &Object::Reference(content_id) => {
                                if let Ok(content) = doc.get_object(content_id) {
                                    self.process_content(content, doc, &mut buffer)?;
                                }
                            },
                            // Array of content objects
                            &Object::Array(ref content_ids) => {
                                for content_id in content_ids {
                                    if let Object::Reference(id) = *content_id {
                                        if let Ok(content) = doc.get_object(id) {
                                            self.process_content(content, doc, &mut buffer)?;
                                        }
                                    }
                                }
                            },
                            _ => {},
                        }
                    },
                    _ => {},
                }
            }
        }
        
        // Clean up the extracted text
        let clean_text = self.clean_text(&buffer);
        
        Ok(clean_text)
    }
    
    /// Process content streams and extract text
    fn process_content(&self, content: &Object, doc: &Document, buffer: &mut String) -> ParserResult<()> {
        if let Ok(data) = doc.get_stream_content(content) {
            // Convert the content stream to a string
            let content_str = String::from_utf8_lossy(&data);
            
            // Use regex to find text sections (enclosed in BT...ET)
            let bt_et_regex = Regex::new(r"BT\s*(.*?)\s*ET").map_err(|e| {
                ParserError::TextExtraction(format!("Regex error: {}", e))
            })?;
            
            for cap in bt_et_regex.captures_iter(&content_str) {
                if let Some(text_section) = cap.get(1) {
                    // Extract text positions and strings from the text section
                    self.extract_text_strings(text_section.as_str(), buffer)?;
                }
            }
        }
        
        Ok(())
    }
    
    /// Extract text strings from a PDF content stream
    fn extract_text_strings(&self, text_section: &str, buffer: &mut String) -> ParserResult<()> {
        // Find Tj and TJ operators which define text strings
        let tj_regex = Regex::new(r"\(([^)]*)\)\s*Tj").map_err(|e| {
            ParserError::TextExtraction(format!("Regex error: {}", e))
        })?;
        
        let tj_multi_regex = Regex::new(r"\[(.*?)\]\s*TJ").map_err(|e| {
            ParserError::TextExtraction(format!("Regex error: {}", e))
        })?;
        
        // Process Tj operators (simple text)
        for cap in tj_regex.captures_iter(text_section) {
            if let Some(text) = cap.get(1) {
                buffer.push_str(&self.unescape_pdf_string(text.as_str()));
                buffer.push(' ');
            }
        }
        
        // Process TJ operators (text with kerning/spacing information)
        for cap in tj_multi_regex.captures_iter(text_section) {
            if let Some(text_array) = cap.get(1) {
                // Extract strings from array
                let string_regex = Regex::new(r"\(([^)]*)\)").map_err(|e| {
                    ParserError::TextExtraction(format!("Regex error: {}", e))
                })?;
                
                for string_cap in string_regex.captures_iter(text_array.as_str()) {
                    if let Some(text) = string_cap.get(1) {
                        buffer.push_str(&self.unescape_pdf_string(text.as_str()));
                        buffer.push(' ');
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Unescape a PDF string
    fn unescape_pdf_string(&self, text: &str) -> String {
        // Basic unescaping for PDF strings
        text.replace("\\(", "(")
            .replace("\\)", ")")
            .replace("\\\\", "\\")
    }
    
    /// Clean up the extracted text
    fn clean_text(&self, text: &str) -> String {
        lazy_static! {
            // Patterns for cleanup
            static ref DUPLICATE_SPACES: Regex = Regex::new(r"\s{2,}").unwrap();
            static ref DUPLICATE_NEWLINES: Regex = Regex::new(r"\n{3,}").unwrap();
        }
        
        // Replace duplicate spaces with a single space
        let cleaned = DUPLICATE_SPACES.replace_all(text, " ");
        
        // Replace more than two consecutive newlines with two
        let cleaned = DUPLICATE_NEWLINES.replace_all(&cleaned, "\n\n");
        
        cleaned.to_string()
    }
    
    /// Extract sections from text content
    fn extract_sections(&self, text: &str) -> Vec<PdfSection> {
        lazy_static! {
            // Pattern to match potential section headings
            static ref SECTION_HEADING: Regex = Regex::new(
                r"(?m)^\s*(\d+\.(?:\d+\.)*\s+)?([A-Z][A-Za-z0-9\s\-:,.]+)$"
            ).unwrap();
        }
        
        let mut sections = Vec::new();
        let mut current_section = PdfSection {
            heading: "Introduction".to_string(),
            content: String::new(),
            level: 1,
        };
        
        let lines: Vec<&str> = text.lines().collect();
        let mut i = 0;
        
        while i < lines.len() {
            let line = lines[i].trim();
            
            // Check if the line matches a section heading pattern
            if let Some(caps) = SECTION_HEADING.captures(line) {
                // If we've accumulated content in the current section, add it
                if !current_section.content.trim().is_empty() {
                    sections.push(current_section);
                }
                
                // Determine the section level based on numbering
                let level = if let Some(numbering) = caps.get(1) {
                    let dots = numbering.as_str().matches('.').count();
                    dots as u8
                } else {
                    1
                };
                
                // Create a new section
                current_section = PdfSection {
                    heading: caps.get(2).map_or(line, |m| m.as_str()).trim().to_string(),
                    content: String::new(),
                    level,
                };
            } else {
                // Add the line to the current section content
                current_section.content.push_str(line);
                current_section.content.push('\n');
            }
            
            i += 1;
        }
        
        // Add the last section if it has content
        if !current_section.content.trim().is_empty() {
            sections.push(current_section);
        }
        
        sections
    }
    
    /// Extract references from text content
    fn extract_references(&self, text: &str) -> Vec<String> {
        lazy_static! {
            // Pattern to identify the references section
            static ref REFERENCES_SECTION: Regex = Regex::new(
                r"(?i)(References|Bibliography|Works Cited)[\s\n]+(.+?)(?:\n\n\n|\n\n\s*\d|\z)"
            ).unwrap();
            
            // Pattern to identify individual references
            static ref REFERENCE_ITEM: Regex = Regex::new(
                r"(?m)^\s*(?:\[(\d+)\]|\[([A-Za-z]+\d*)\]|(\d+)\.)\s+(.+?)(?=\n\s*(?:\[\d+\]|\[[A-Za-z]+\d*\]|\d+\.)\s+|\z)"
            ).unwrap();
        }
        
        let mut references = Vec::new();
        
        // Find the references section
        if let Some(caps) = REFERENCES_SECTION.captures(text) {
            if let Some(ref_text) = caps.get(2) {
                // Extract individual references
                for cap in REFERENCE_ITEM.captures_iter(ref_text.as_str()) {
                    let ref_text = cap.get(4).map_or("", |m| m.as_str()).trim();
                    if !ref_text.is_empty() {
                        references.push(ref_text.to_string());
                    }
                }
            }
        }
        
        references
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    
    fn get_test_config() -> PdfConfig {
        PdfConfig {
            extract_sections: true,
            extract_references: true,
        }
    }
} 