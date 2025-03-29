use crate::CloudflareError;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use lazy_static::lazy_static;

lazy_static! {
    static ref IUAM_REGEX: Regex = Regex::new(r#"name="jschl_vc" value="([^"]+)"#).unwrap();
    static ref PASS_REGEX: Regex = Regex::new(r#"name="pass" value="([^"]+)"#).unwrap();
    static ref FORM_ACTION_REGEX: Regex = Regex::new(r#"id="challenge-form" action="([^"]+)"#).unwrap();
    static ref JS_SCRIPT_REGEX: Regex = Regex::new(r#"setTimeout\(function\(\){\s+(var (?:s,t,o,p,b,r,e,a,k,i,n,g|t,r,a,f)).+[\r\n\s]+(.+a\.value =.+)\r?\n"#).unwrap();
    static ref RECAPTCHA_SITEKEY_REGEX: Regex = Regex::new(r#"data-sitekey="([^"]+)"#).unwrap();
    static ref TURNSTILE_SITEKEY_REGEX: Regex = Regex::new(r#"data-sitekey="([^"]+)"#).unwrap();
}

/// Types of Cloudflare challenges
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ChallengeType {
    /// I'm Under Attack Mode challenge
    IUAM,
    /// Standard CAPTCHA challenge
    Captcha,
    /// Cloudflare Turnstile challenge
    Turnstile,
    /// hCaptcha challenge
    HCaptcha,
    /// Custom challenge type
    Custom(String),
}

/// Cloudflare challenge parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Challenge {
    /// The type of challenge
    pub challenge_type: ChallengeType,
    /// URL of the challenge page
    pub url: String,
    /// Form action URL
    pub form_action: Option<String>,
    /// jschl_vc value
    pub jschl_vc: Option<String>,
    /// pass value
    pub pass: Option<String>,
    /// JavaScript challenge script
    pub js_script: Option<String>,
    /// Ray ID
    pub ray_id: Option<String>,
    /// CAPTCHA site key
    pub sitekey: Option<String>,
    /// Other parameters
    pub params: HashMap<String, String>,
}

/// Cloudflare challenge solution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeSolution {
    /// The type of challenge
    pub challenge_type: ChallengeType,
    /// URL to submit the solution to
    pub submit_url: String,
    /// Solution parameters
    pub params: HashMap<String, String>,
    /// Cookies to set
    pub cookies: HashMap<String, String>,
}

/// Extract an IUAM challenge from HTML
pub fn extract_iuam_challenge(html: &str) -> Result<Challenge, CloudflareError> {
    // Extract jschl_vc
    let jschl_vc = IUAM_REGEX
        .captures(html)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| CloudflareError::ChallengeDetected("Could not find jschl_vc".to_string()))?;
    
    // Extract pass
    let pass = PASS_REGEX
        .captures(html)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| CloudflareError::ChallengeDetected("Could not find pass".to_string()))?;
    
    // Extract form action
    let form_action = FORM_ACTION_REGEX
        .captures(html)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| CloudflareError::ChallengeDetected("Could not find form action".to_string()))?;
    
    // Extract JavaScript challenge
    let js_script = JS_SCRIPT_REGEX
        .captures(html)
        .and_then(|caps| {
            if caps.len() >= 3 {
                Some(format!("{}{}", caps.get(1)?.as_str(), caps.get(2)?.as_str()))
            } else {
                None
            }
        })
        .ok_or_else(|| CloudflareError::ChallengeDetected("Could not find JavaScript challenge".to_string()))?;
    
    // Extract Ray ID
    let ray_id = html
        .lines()
        .find(|line| line.contains("Ray ID:"))
        .and_then(|line| {
            let parts: Vec<&str> = line.split("Ray ID:").collect();
            if parts.len() >= 2 {
                Some(parts[1].trim().to_string())
            } else {
                None
            }
        });
    
    Ok(Challenge {
        challenge_type: ChallengeType::IUAM,
        url: "".to_string(), // Will be set by the caller
        form_action: Some(form_action),
        jschl_vc: Some(jschl_vc),
        pass: Some(pass),
        js_script: Some(js_script),
        ray_id,
        sitekey: None,
        params: HashMap::new(),
    })
}

/// Extract a CAPTCHA challenge from HTML
pub fn extract_captcha_challenge(html: &str) -> Result<Challenge, CloudflareError> {
    // Extract sitekey
    let sitekey = RECAPTCHA_SITEKEY_REGEX
        .captures(html)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| CloudflareError::ChallengeDetected("Could not find reCAPTCHA sitekey".to_string()))?;
    
    // Extract form action
    let form_action = FORM_ACTION_REGEX
        .captures(html)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| CloudflareError::ChallengeDetected("Could not find form action".to_string()))?;
    
    // Extract Ray ID
    let ray_id = html
        .lines()
        .find(|line| line.contains("Ray ID:"))
        .and_then(|line| {
            let parts: Vec<&str> = line.split("Ray ID:").collect();
            if parts.len() >= 2 {
                Some(parts[1].trim().to_string())
            } else {
                None
            }
        });
    
    Ok(Challenge {
        challenge_type: ChallengeType::Captcha,
        url: "".to_string(), // Will be set by the caller
        form_action: Some(form_action),
        jschl_vc: None,
        pass: None,
        js_script: None,
        ray_id,
        sitekey: Some(sitekey),
        params: HashMap::new(),
    })
}

/// Extract a Turnstile challenge from HTML
pub fn extract_turnstile_challenge(html: &str) -> Result<Challenge, CloudflareError> {
    // Extract sitekey
    let sitekey = TURNSTILE_SITEKEY_REGEX
        .captures(html)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| CloudflareError::ChallengeDetected("Could not find Turnstile sitekey".to_string()))?;
    
    // Extract form action
    let form_action = FORM_ACTION_REGEX
        .captures(html)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
        .ok_or_else(|| CloudflareError::ChallengeDetected("Could not find form action".to_string()))?;
    
    // Extract Ray ID
    let ray_id = html
        .lines()
        .find(|line| line.contains("Ray ID:"))
        .and_then(|line| {
            let parts: Vec<&str> = line.split("Ray ID:").collect();
            if parts.len() >= 2 {
                Some(parts[1].trim().to_string())
            } else {
                None
            }
        });
    
    Ok(Challenge {
        challenge_type: ChallengeType::Turnstile,
        url: "".to_string(), // Will be set by the caller
        form_action: Some(form_action),
        jschl_vc: None,
        pass: None,
        js_script: None,
        ray_id,
        sitekey: Some(sitekey),
        params: HashMap::new(),
    })
}

/// Solve an IUAM challenge
pub fn solve_iuam_challenge(challenge: &Challenge, domain: &str) -> Result<ChallengeSolution, CloudflareError> {
    if challenge.challenge_type != ChallengeType::IUAM {
        return Err(CloudflareError::ChallengeSolvingFailed("Not an IUAM challenge".to_string()));
    }
    
    let js_script = challenge.js_script.as_ref()
        .ok_or_else(|| CloudflareError::ChallengeSolvingFailed("No JavaScript challenge script".to_string()))?;
    
    let jschl_vc = challenge.jschl_vc.as_ref()
        .ok_or_else(|| CloudflareError::ChallengeSolvingFailed("No jschl_vc value".to_string()))?;
    
    let pass = challenge.pass.as_ref()
        .ok_or_else(|| CloudflareError::ChallengeSolvingFailed("No pass value".to_string()))?;
    
    let form_action = challenge.form_action.as_ref()
        .ok_or_else(|| CloudflareError::ChallengeSolvingFailed("No form action".to_string()))?;
    
    // Add the domain length to the JavaScript script
    // This is a simplified solution - a real one would execute the JS
    let domain_len = domain.chars().count();
    let js_solution = format!("answer = {}; answer", domain_len);
    
    // In a real implementation, we would execute the JavaScript challenge
    // using a JavaScript engine. Here, we just use a placeholder.
    let js_result = "1234"; // Placeholder
    
    let mut params = HashMap::new();
    params.insert("jschl_vc".to_string(), jschl_vc.clone());
    params.insert("pass".to_string(), pass.clone());
    params.insert("jschl_answer".to_string(), js_result.to_string());
    
    let submit_url = if form_action.starts_with("http") {
        form_action.clone()
    } else {
        format!("https://{}{}", domain, form_action)
    };
    
    Ok(ChallengeSolution {
        challenge_type: ChallengeType::IUAM,
        submit_url,
        params,
        cookies: HashMap::new(),
    })
}

/// Main function to solve a Cloudflare challenge
pub fn solve_challenge(challenge: &Challenge, domain: &str) -> Result<ChallengeSolution, CloudflareError> {
    match challenge.challenge_type {
        ChallengeType::IUAM => solve_iuam_challenge(challenge, domain),
        ChallengeType::Captcha | ChallengeType::Turnstile | ChallengeType::HCaptcha => {
            Err(CloudflareError::CaptchaRequired("CAPTCHA solving not implemented in this function".to_string()))
        }
        ChallengeType::Custom(_) => {
            Err(CloudflareError::ChallengeSolvingFailed("Custom challenge type not supported".to_string()))
        }
    }
} 