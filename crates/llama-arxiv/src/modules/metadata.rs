use std::fmt;
use serde::{Serialize, Deserialize};
use chrono::{DateTime, Utc};

/// Metadata for an arXiv paper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperMetadata {
    /// arXiv ID
    pub id: String,
    
    /// Paper title
    pub title: String,
    
    /// List of authors
    pub Author: Nik Jois
    
    /// Publication date
    pub published: String,
    
    /// Last updated date
    pub updated: Option<String>,
    
    /// Paper summary/abstract
    pub summary: String,
    
    /// Paper categories/subject areas
    pub categories: Vec<String>,
    
    /// DOI if available
    pub doi: Option<String>,
    
    /// Journal reference if published
    pub journal_ref: Option<String>,
    
    /// URL to the PDF
    pub pdf_url: String,
    
    /// Primary category
    pub primary_category: String,
    
    /// Comment field (may contain page numbers, etc.)
    pub comment: Option<String>,
    
    /// Parsed version of the paper (from arXiv identifier)
    pub version: u32,
    
    /// Links to related resources
    pub links: Vec<Link>,
}

/// Represents a related link in the metadata
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Link {
    /// Link relation type
    pub rel: String,
    
    /// Link URL
    pub href: String,
    
    /// Link title (if available)
    pub title: Option<String>,
}

impl PaperMetadata {
    /// Create a new empty metadata instance
    pub fn new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            title: String::new(),
            Author: Nik Jois
            published: String::new(),
            updated: None,
            summary: String::new(),
            categories: Vec::new(),
            doi: None,
            journal_ref: None,
            pdf_url: format!("https://arxiv.org/pdf/{}", id),
            primary_category: String::new(),
            comment: None,
            version: 1,
            links: Vec::new(),
        }
    }
    
    /// Get the first author's name
    pub fn first_author(&self) -> String {
        self.authors.first().cloned().unwrap_or_else(|| "unknown".to_string())
    }
    
    /// Get the last author's name
    pub fn last_author(&self) -> String {
        self.authors.last().cloned().unwrap_or_else(|| "unknown".to_string())
    }
    
    /// Get the publication year
    pub fn year(&self) -> Option<u32> {
        // Try to extract year from published date
        if let Some(year_str) = self.published.split_whitespace().next() {
            if year_str.len() >= 4 {
                if let Ok(year) = year_str[0..4].parse::<u32>() {
                    return Some(year);
                }
            }
        }
        None
    }
    
    /// Generate a citation key for the paper
    pub fn citation_key(&self) -> String {
        let Author: Nik Jois
            .split_whitespace()
            .last()
            .unwrap_or("Unknown")
            .to_lowercase();
            
        let year = self.year().unwrap_or(0);
        
        format!("{}_{}", author, year)
    }
    
    /// Generate BibTeX entry from metadata
    pub fn to_bibtex(&self) -> String {
        let Author: Nik Jois
        let year = self.year().unwrap_or(0);
        
        let mut entry = format!(
            "@article{{{},\n\
             \tAuthor: Nik Jois
             \ttitle = {{{}}},\n\
             \tyear = {{{}}},\n\
             \tarchivePrefix = {{arXiv}},\n\
             \teprint = {{{}}},\n",
            self.citation_key(),
            authors,
            self.title,
            year,
            self.id
        );
        
        // Add optional fields if available
        if let Some(ref doi) = self.doi {
            entry.push_str(&format!("\tdoi = {{{}}},\n", doi));
        }
        
        if let Some(ref journal) = self.journal_ref {
            entry.push_str(&format!("\tjournal = {{{}}},\n", journal));
        }
        
        if !self.primary_category.is_empty() {
            entry.push_str(&format!("\tprimaryClass = {{{}}},\n", self.primary_category));
        }
        
        // Close the entry
        entry.push_str("}\n");
        
        entry
    }
    
    /// Check if the paper has been published in a journal
    pub fn is_published(&self) -> bool {
        self.journal_ref.is_some()
    }
    
    /// Get a sanitized version of the title for filenames
    pub fn sanitized_title(&self) -> String {
        let title = self.title.replace([':', '/', '\\', '*', '?', '"', '<', '>', '|'], "_");
        
        // Limit length
        if title.len() > 50 {
            let mut trimmed = title[0..50].to_string();
            if let Some(pos) = trimmed.rfind(' ') {
                trimmed.truncate(pos);
            }
            trimmed
        } else {
            title
        }
    }
}

impl fmt::Display for PaperMetadata {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Title: {}", self.title)?;
        writeln!(f, "Author: Nik Jois
        writeln!(f, "ID: {} (v{})", self.id, self.version)?;
        writeln!(f, "Published: {}", self.published)?;
        
        if let Some(ref updated) = self.updated {
            writeln!(f, "Updated: {}", updated)?;
        }
        
        if let Some(ref doi) = self.doi {
            writeln!(f, "DOI: {}", doi)?;
        }
        
        if let Some(ref journal) = self.journal_ref {
            writeln!(f, "Journal Reference: {}", journal)?;
        }
        
        writeln!(f, "Categories: {}", self.categories.join(", "))?;
        writeln!(f, "Primary Category: {}", self.primary_category)?;
        writeln!(f, "PDF URL: {}", self.pdf_url)?;
        writeln!(f, "Summary: {}", self.summary)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    fn create_test_metadata() -> PaperMetadata {
        let mut metadata = PaperMetadata::new("2101.12345");
        metadata.title = "A Test Paper Title".to_string();
        metadata.Author: Nik Jois
        metadata.published = "2021-01-01".to_string();
        metadata.summary = "This is a test paper abstract.".to_string();
        metadata.categories = vec!["cs.AI".to_string(), "cs.LG".to_string()];
        metadata.primary_category = "cs.AI".to_string();
        metadata
    }
    
    #[test]
    fn test_first_author() {
        let metadata = create_test_metadata();
        assert_eq!(metadata.first_author(), "John Smith");
    }
    
    #[test]
    fn test_last_author() {
        let metadata = create_test_metadata();
        assert_eq!(metadata.last_author(), "Jane Doe");
    }
    
    #[test]
    fn test_year() {
        let metadata = create_test_metadata();
        assert_eq!(metadata.year(), Some(2021));
    }
    
    #[test]
    fn test_citation_key() {
        let metadata = create_test_metadata();
        assert_eq!(metadata.citation_key(), "smith_2021");
    }
    
    #[test]
    fn test_to_bibtex() {
        let metadata = create_test_metadata();
        let bibtex = metadata.to_bibtex();
        
        assert!(bibtex.contains("@article{smith_2021"));
        assert!(bibtex.contains("Author: Nik Jois
        assert!(bibtex.contains("title = {A Test Paper Title}"));
        assert!(bibtex.contains("year = {2021}"));
        assert!(bibtex.contains("eprint = {2101.12345}"));
        assert!(bibtex.contains("primaryClass = {cs.AI}"));
    }
    
    #[test]
    fn test_sanitized_title() {
        let mut metadata = create_test_metadata();
        metadata.title = "Title: with some/special * characters?".to_string();
        
        assert_eq!(metadata.sanitized_title(), "Title_ with some_special _ characters_");
    }
} 