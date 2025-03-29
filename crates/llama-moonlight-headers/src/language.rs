use rand::prelude::*;

/// Common language codes
pub const LANG_EN_US: &str = "en-US";
pub const LANG_EN_GB: &str = "en-GB";
pub const LANG_FR_FR: &str = "fr-FR";
pub const LANG_DE_DE: &str = "de-DE";
pub const LANG_ES_ES: &str = "es-ES";
pub const LANG_IT_IT: &str = "it-IT";
pub const LANG_PT_BR: &str = "pt-BR";
pub const LANG_RU_RU: &str = "ru-RU";
pub const LANG_JA_JP: &str = "ja-JP";
pub const LANG_ZH_CN: &str = "zh-CN";
pub const LANG_ZH_TW: &str = "zh-TW";
pub const LANG_KO_KR: &str = "ko-KR";
pub const LANG_AR_SA: &str = "ar-SA";
pub const LANG_NL_NL: &str = "nl-NL";
pub const LANG_PL_PL: &str = "pl-PL";
pub const LANG_TR_TR: &str = "tr-TR";
pub const LANG_SV_SE: &str = "sv-SE";
pub const LANG_NO_NO: &str = "no-NO";
pub const LANG_DA_DK: &str = "da-DK";
pub const LANG_FI_FI: &str = "fi-FI";

/// Structure representing a language with region and weight
#[derive(Debug, Clone, PartialEq)]
pub struct Language {
    /// Language code (e.g., "en")
    pub code: String,
    
    /// Region code (e.g., "US")
    pub region: Option<String>,
    
    /// Weight (quality value) for Accept-Language
    pub weight: Option<f32>,
}

impl Language {
    /// Create a new Language
    pub fn new(code: &str, region: Option<&str>, weight: Option<f32>) -> Self {
        Self {
            code: code.to_string(),
            region: region.map(|r| r.to_string()),
            weight: weight,
        }
    }
    
    /// Create a Language from a string like "en-US" or "en-US;q=0.8"
    pub fn from_str(s: &str) -> Option<Self> {
        let parts: Vec<&str> = s.split(';').collect();
        let lang_region = parts[0].trim();
        
        let weight = if parts.len() > 1 {
            if let Some(q_part) = parts[1].trim().strip_prefix("q=") {
                q_part.parse::<f32>().ok()
            } else {
                None
            }
        } else {
            None
        };
        
        let lang_parts: Vec<&str> = lang_region.split('-').collect();
        let (code, region) = match lang_parts.len() {
            1 => (lang_parts[0], None),
            2 => (lang_parts[0], Some(lang_parts[1])),
            _ => return None,
        };
        
        Some(Self::new(code, region, weight))
    }
    
    /// Format the language as a string for Accept-Language header
    pub fn to_string(&self) -> String {
        let lang_part = match &self.region {
            Some(region) => format!("{}-{}", self.code, region),
            None => self.code.clone(),
        };
        
        match self.weight {
            Some(weight) => format!("{};q={:.1}", lang_part, weight),
            None => lang_part,
        }
    }
}

impl std::fmt::Display for Language {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// Get a list of common languages
pub fn common_languages() -> Vec<&'static str> {
    vec![
        LANG_EN_US,
        LANG_EN_GB,
        LANG_FR_FR,
        LANG_DE_DE,
        LANG_ES_ES,
        LANG_IT_IT,
        LANG_PT_BR,
        LANG_RU_RU,
        LANG_JA_JP,
        LANG_ZH_CN,
    ]
}

/// Get a list of all supported languages
pub fn all_languages() -> Vec<&'static str> {
    vec![
        LANG_EN_US,
        LANG_EN_GB,
        LANG_FR_FR,
        LANG_DE_DE,
        LANG_ES_ES,
        LANG_IT_IT,
        LANG_PT_BR,
        LANG_RU_RU,
        LANG_JA_JP,
        LANG_ZH_CN,
        LANG_ZH_TW,
        LANG_KO_KR,
        LANG_AR_SA,
        LANG_NL_NL,
        LANG_PL_PL,
        LANG_TR_TR,
        LANG_SV_SE,
        LANG_NO_NO,
        LANG_DA_DK,
        LANG_FI_FI,
    ]
}

/// Generate a random Accept-Language header
pub fn random_language() -> String {
    let mut rng = rand::thread_rng();
    
    // Choose a primary language
    let primary_lang = common_languages()[rng.gen_range(0..common_languages().len())];
    
    // 50% chance to include a second language
    let include_second = rng.gen_bool(0.5);
    
    if include_second {
        // Choose a secondary language different from the primary
        let mut secondary_langs = common_languages();
        secondary_langs.retain(|&lang| lang != primary_lang);
        let secondary_lang = secondary_langs[rng.gen_range(0..secondary_langs.len())];
        
        format!("{},{}q=0.{}", primary_lang, secondary_lang, rng.gen_range(5..10))
    } else {
        primary_lang.to_string()
    }
}

/// Generate a random Accept-Language header for Safari (includes more variants)
pub fn random_safari_language() -> String {
    let mut rng = rand::thread_rng();
    
    // Choose a primary language
    let primary_lang = common_languages()[rng.gen_range(0..common_languages().len())];
    
    // For Safari, often include the generic language code as well
    let primary_parts: Vec<&str> = primary_lang.split('-').collect();
    let primary_code = primary_parts[0];
    
    format!("{},{};q=0.9,*;q=0.8", primary_lang, primary_code)
}

/// Generate a realistic Accept-Language header for the given primary language
pub fn generate_accept_language(primary_lang: &str) -> String {
    let mut rng = rand::thread_rng();
    
    // Extract language code
    let primary_parts: Vec<&str> = primary_lang.split('-').collect();
    let primary_code = primary_parts[0];
    
    // Add the generic language code with a high weight
    let mut languages = vec![
        Language::new(primary_parts[0], primary_parts.get(1).copied(), None),
        Language::new(primary_parts[0], None, Some(0.9)),
    ];
    
    // 70% chance to add English if primary is not English
    if primary_code != "en" && rng.gen_bool(0.7) {
        languages.push(Language::new("en", Some("US"), Some(0.8)));
    }
    
    // 30% chance to add a third random language
    if rng.gen_bool(0.3) {
        let mut all_langs = common_languages();
        all_langs.retain(|&lang| !lang.starts_with(primary_code) && !lang.starts_with("en"));
        
        if !all_langs.is_empty() {
            let random_lang = all_langs[rng.gen_range(0..all_langs.len())];
            let random_parts: Vec<&str> = random_lang.split('-').collect();
            
            languages.push(Language::new(random_parts[0], random_parts.get(1).copied(), Some(0.7)));
        }
    }
    
    // 20% chance to add wildcard with low weight
    if rng.gen_bool(0.2) {
        languages.push(Language::new("*", None, Some(0.5)));
    }
    
    // Format the languages into an Accept-Language string
    languages.iter()
        .map(|l| l.to_string())
        .collect::<Vec<String>>()
        .join(",")
}

/// Get a map of languages with their English names
pub fn language_names() -> std::collections::HashMap<&'static str, &'static str> {
    let mut names = std::collections::HashMap::new();
    names.insert(LANG_EN_US, "English (United States)");
    names.insert(LANG_EN_GB, "English (United Kingdom)");
    names.insert(LANG_FR_FR, "French (France)");
    names.insert(LANG_DE_DE, "German (Germany)");
    names.insert(LANG_ES_ES, "Spanish (Spain)");
    names.insert(LANG_IT_IT, "Italian (Italy)");
    names.insert(LANG_PT_BR, "Portuguese (Brazil)");
    names.insert(LANG_RU_RU, "Russian (Russia)");
    names.insert(LANG_JA_JP, "Japanese (Japan)");
    names.insert(LANG_ZH_CN, "Chinese (Simplified, China)");
    names.insert(LANG_ZH_TW, "Chinese (Traditional, Taiwan)");
    names.insert(LANG_KO_KR, "Korean (Korea)");
    names.insert(LANG_AR_SA, "Arabic (Saudi Arabia)");
    names.insert(LANG_NL_NL, "Dutch (Netherlands)");
    names.insert(LANG_PL_PL, "Polish (Poland)");
    names.insert(LANG_TR_TR, "Turkish (Turkey)");
    names.insert(LANG_SV_SE, "Swedish (Sweden)");
    names.insert(LANG_NO_NO, "Norwegian (Norway)");
    names.insert(LANG_DA_DK, "Danish (Denmark)");
    names.insert(LANG_FI_FI, "Finnish (Finland)");
    names
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_language_new() {
        let lang = Language::new("en", Some("US"), Some(0.8));
        assert_eq!(lang.code, "en");
        assert_eq!(lang.region, Some("US".to_string()));
        assert_eq!(lang.weight, Some(0.8));
    }
    
    #[test]
    fn test_language_from_str() {
        let lang = Language::from_str("en-US").unwrap();
        assert_eq!(lang.code, "en");
        assert_eq!(lang.region, Some("US".to_string()));
        assert_eq!(lang.weight, None);
        
        let lang = Language::from_str("fr-FR;q=0.8").unwrap();
        assert_eq!(lang.code, "fr");
        assert_eq!(lang.region, Some("FR".to_string()));
        assert_eq!(lang.weight, Some(0.8));
        
        let lang = Language::from_str("de;q=0.5").unwrap();
        assert_eq!(lang.code, "de");
        assert_eq!(lang.region, None);
        assert_eq!(lang.weight, Some(0.5));
    }
    
    #[test]
    fn test_language_to_string() {
        let lang = Language::new("en", Some("US"), None);
        assert_eq!(lang.to_string(), "en-US");
        
        let lang = Language::new("fr", Some("FR"), Some(0.8));
        assert_eq!(lang.to_string(), "fr-FR;q=0.8");
        
        let lang = Language::new("de", None, Some(0.5));
        assert_eq!(lang.to_string(), "de;q=0.5");
    }
    
    #[test]
    fn test_random_language() {
        let lang = random_language();
        assert!(!lang.is_empty());
    }
    
    #[test]
    fn test_random_safari_language() {
        let lang = random_safari_language();
        assert!(!lang.is_empty());
        assert!(lang.contains(";q=0.9"));
        assert!(lang.contains("*;q=0.8"));
    }
    
    #[test]
    fn test_generate_accept_language() {
        let lang = generate_accept_language(LANG_EN_US);
        assert!(!lang.is_empty());
        assert!(lang.starts_with("en-US"));
        
        let lang = generate_accept_language(LANG_FR_FR);
        assert!(!lang.is_empty());
        assert!(lang.starts_with("fr-FR"));
    }
} 