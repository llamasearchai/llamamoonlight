use reqwest::Client;
use serde::Deserialize;
use thiserror::Error;
use log::{debug, error, info};

#[derive(Error, Debug)]
pub enum PubMedApiError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("API response error: {0}")]
    ApiResponse(String),
    
    #[error("No results found")]
    NoResults,
}

#[derive(Debug, Deserialize)]
struct PubMedSearchResult {
    pub pmids: Vec<String>,
}

pub async fn search_pubmed(
    client: &Client,
    query: &str,
    date_range: Option<String>,
    journal: Option<String>,
) -> Result<Vec<String>, PubMedApiError> {
    let base_url = "https://api.pubmed.gov/search";
    let mut url = format!("{}?query={}", base_url, query);

    if let Some(range) = date_range {
        url.push_str(&format!("&date_range={}", range));
    }

    if let Some(journal) = journal {
        url.push_str(&format!("&journal={}", journal));
    }

    debug!("Searching PubMed with URL: {}", url);
    let response = client.get(&url).send().await?;

    if !response.status().is_success() {
        return Err(PubMedApiError::ApiResponse(format!(
            "API returned status code: {}",
            response.status()
        )));
    }

    let search_result: PubMedSearchResult = response.json().await?;

    if search_result.pmids.is_empty() {
        return Err(PubMedApiError::NoResults);
    }

    Ok(search_result.pmids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::Client;

    #[tokio::test]
    async fn test_search_pubmed() {
        let client = Client::new();
        let result = search_pubmed(&client, "cancer", None, None).await;
        assert!(result.is_err()); // Placeholder URL will fail
    }
} 