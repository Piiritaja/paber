use anyhow::{Context, Result};
use base64::prelude::*;
use reqwest::blocking::Client; 
use serde::Deserialize;
use serde_json::json;
use std::fs;
use std::env;

#[derive(Debug, Deserialize)]
struct GeminiResponse { candidates: Option<Vec<Candidate>> }
#[derive(Debug, Deserialize)]
struct Candidate { content: Content }
#[derive(Debug, Deserialize)]
struct Content { parts: Vec<Part> }
#[derive(Debug, Deserialize)]
struct Part {
    #[serde(rename = "inlineData")]
    inline_data: Option<InlineData>,
}
#[derive(Debug, Deserialize)]
struct InlineData { data: String }

pub struct WallpaperTool {
    client: Client,
    api_key: String,
}

impl WallpaperTool {
    pub fn new() -> Result<Self> {
        let api_key = env::var("GEMINI_API_KEY")
            .context("GEMINI_API_KEY must be set")?;
        
        Ok(Self {
            client: Client::new(),
            api_key,
        })
    }

    pub fn generate_online(&self, prompt: &str, output_path: &str) -> Result<()> {
        println!("Sending prompt to Google AI Studio...");

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.5-flash-image:generateContent?key={}", 
            self.api_key
        );

        let payload = json!({
            "contents": [{
                "parts": [{ "text": prompt }]
            }],
            "generationConfig": {
                "responseModalities": ["IMAGE"] 
            }
        });

        let res = self.client.post(&url)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .context("Failed to contact Google API")?;

        if !res.status().is_success() {
            anyhow::bail!("API Error: {}", res.text()?);
        }

        let response_body: GeminiResponse = res.json()
            .context("Failed to parse JSON")?;

        if let Some(candidates) = response_body.candidates {
            if let Some(first) = candidates.first() {
                for part in &first.content.parts {
                    if let Some(inline_data) = &part.inline_data {
                        println!("Image received! Decoding...");
                        
                        let clean_b64 = inline_data.data.replace('\n', "");
                        let image_bytes = BASE64_STANDARD.decode(&clean_b64)
                            .context("Failed to decode base64 image data")?;
                        fs::write(output_path, image_bytes)?;
                        println!("Image saved to {}", output_path);
                        return Ok(());
                    }
                }
            }
        }
        anyhow::bail!("No image found in response");
    }
}
