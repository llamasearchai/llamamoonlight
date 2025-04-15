use crate::MlxError;
use image::{DynamicImage, GenericImageView, ImageBuffer};
use std::{path::Path, io::Read};

/// Load an image from a file
pub fn load_image(path: &Path) -> Result<DynamicImage, MlxError> {
    image::open(path)
        .map_err(|e| MlxError::ImageProcessing(format!("Failed to load image: {}", e)))
}

/// Save an image to a file
pub fn save_image(image: &DynamicImage, path: &Path) -> Result<(), MlxError> {
    image.save(path)
        .map_err(|e| MlxError::ImageProcessing(format!("Failed to save image: {}", e)))
}

/// Resize an image to a specified width and height
pub fn resize_image(image: &DynamicImage, width: u32, height: u32) -> DynamicImage {
    image.resize_exact(width, height, image::imageops::FilterType::Lanczos3)
}

/// Convert an image to grayscale
pub fn grayscale_image(image: &DynamicImage) -> DynamicImage {
    image.grayscale()
}

/// Crop an image to a specified region
pub fn crop_image(image: &DynamicImage, x: u32, y: u32, width: u32, height: u32) -> Result<DynamicImage, MlxError> {
    if x + width > image.width() || y + height > image.height() {
        return Err(MlxError::ImageProcessing(format!(
            "Crop region ({}, {}, {}, {}) is outside image dimensions ({}x{})",
            x, y, width, height, image.width(), image.height()
        )));
    }
    
    Ok(image.crop_imm(x, y, width, height))
}

/// Encode an image to base64
pub fn encode_image_base64(image: &DynamicImage, format: &str) -> Result<String, MlxError> {
    let mut buffer = Vec::new();
    
    match format.to_lowercase().as_str() {
        "png" => {
            image.write_to(&mut buffer, image::ImageOutputFormat::Png)
                .map_err(|e| MlxError::ImageProcessing(format!("Failed to encode image as PNG: {}", e)))?;
        }
        "jpeg" | "jpg" => {
            image.write_to(&mut buffer, image::ImageOutputFormat::Jpeg(90))
                .map_err(|e| MlxError::ImageProcessing(format!("Failed to encode image as JPEG: {}", e)))?;
        }
        "webp" => {
            return Err(MlxError::ImageProcessing("WebP encoding not supported".to_string()));
        }
        _ => {
            return Err(MlxError::ImageProcessing(format!("Unsupported image format: {}", format)));
        }
    }
    
    Ok(base64::encode(&buffer))
}

/// Decode an image from base64
pub fn decode_image_base64(data: &str, format: &str) -> Result<DynamicImage, MlxError> {
    let bytes = base64::decode(data)
        .map_err(|e| MlxError::ImageProcessing(format!("Failed to decode base64: {}", e)))?;
    
    image::load_from_memory(&bytes)
        .map_err(|e| MlxError::ImageProcessing(format!("Failed to decode image: {}", e)))
}

/// Extract text from an image (placeholder - in a real implementation, this would use OCR)
pub fn extract_text_from_image(_image: &DynamicImage) -> Result<String, MlxError> {
    Err(MlxError::UnsupportedFeature("OCR is not implemented in this version".to_string()))
}

/// Image normalization for neural network input
pub fn normalize_image(image: &DynamicImage, mean: &[f32], std: &[f32]) -> Result<Vec<f32>, MlxError> {
    if mean.len() != 3 || std.len() != 3 {
        return Err(MlxError::ImageProcessing(
            "Mean and std arrays must have 3 elements (RGB)".to_string()
        ));
    }
    
    let rgb_image = image.to_rgb8();
    let (width, height) = rgb_image.dimensions();
    let mut normalized = Vec::with_capacity((width * height * 3) as usize);
    
    for y in 0..height {
        for x in 0..width {
            let pixel = rgb_image.get_pixel(x, y);
            let r = (pixel[0] as f32 / 255.0 - mean[0]) / std[0];
            let g = (pixel[1] as f32 / 255.0 - mean[1]) / std[1];
            let b = (pixel[2] as f32 / 255.0 - mean[2]) / std[2];
            
            normalized.push(r);
            normalized.push(g);
            normalized.push(b);
        }
    }
    
    Ok(normalized)
}

/// Load a text file
pub fn load_text_file(path: &Path) -> Result<String, MlxError> {
    let mut file = std::fs::File::open(path)
        .map_err(|e| MlxError::IO(e))?;
    
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| MlxError::IO(e))?;
    
    Ok(contents)
}

/// Split a text into chunks of maximum token length (approximation)
pub fn split_text_into_chunks(text: &str, max_tokens: usize) -> Vec<String> {
    // Simple approximation - assume words are ~1 token each
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut chunks = Vec::new();
    let mut current_chunk = Vec::new();
    let mut current_count = 0;
    
    for word in words {
        current_chunk.push(word);
        current_count += 1;
        
        if current_count >= max_tokens {
            chunks.push(current_chunk.join(" "));
            current_chunk.clear();
            current_count = 0;
        }
    }
    
    if !current_chunk.is_empty() {
        chunks.push(current_chunk.join(" "));
    }
    
    chunks
}

/// Calculate image hash for similarity comparison
pub fn image_hash(_image: &DynamicImage) -> Result<String, MlxError> {
    // This is a placeholder - in a real implementation, this would calculate a perceptual hash
    Err(MlxError::UnsupportedFeature("Image hashing is not implemented in this version".to_string()))
}

/// Simple utility to remove HTML tags from text
pub fn strip_html_tags(html: &str) -> String {
    // Very simple HTML tag stripping - not a complete solution
    let mut result = String::new();
    let mut in_tag = false;
    
    for c in html.chars() {
        match c {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(c),
            _ => {}
        }
    }
    
    result
} 