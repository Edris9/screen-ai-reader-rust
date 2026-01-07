use serde_json::json;
use reqwest::blocking::Client;
use std::time::Duration;
use image::DynamicImage;
use image::ImageFormat;
use std::io::Cursor;
use base64::{Engine as _, engine::general_purpose};
use crate::app::OnlineProvider;

// Hjälpfunktion för att göra om bild till Base64
fn image_to_base64(img: &DynamicImage) -> Result<String, String> {
    let mut bytes: Vec<u8> = Vec::new();
    img.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
        .map_err(|_| "Kunde inte bearbeta bilden".to_string())?;
    Ok(general_purpose::STANDARD.encode(&bytes))
}

pub fn send_chat(
    provider: &OnlineProvider,
    api_key: &str,
    model: &str,
    image: Option<&DynamicImage>,
    history: &Vec<(bool, String)>
) -> Result<String, String> {
    
    if api_key.trim().is_empty() {
        return Err("Ingen API-nyckel angiven!".to_string());
    }

    let client = Client::builder()
        .timeout(Duration::from_secs(60))
        .build()
        .map_err(|e| e.to_string())?;

    let base64_image = if let Some(img) = image {
        Some(image_to_base64(img)?)
    } else {
        None
    };

    match provider {
        OnlineProvider::OpenAI | OnlineProvider::Groq | OnlineProvider::Grok => {
            send_openai_style(client, provider, api_key, model, base64_image, history)
        }
        OnlineProvider::Anthropic => {
            send_anthropic(client, api_key, model, base64_image, history)
        }
    }
}

// --- OPENAI / GROQ / GROK LOGIK ---
fn send_openai_style(
    client: Client,
    provider: &OnlineProvider,
    api_key: &str,
    model: &str,
    base64_image: Option<String>,
    history: &Vec<(bool, String)>
) -> Result<String, String> {
    
    let url = match provider {
        OnlineProvider::OpenAI => "https://api.openai.com/v1/chat/completions",
        OnlineProvider::Groq => "https://api.groq.com/openai/v1/chat/completions",
        OnlineProvider::Grok => "https://api.x.ai/v1/chat/completions", // Exempel-URL
        _ => return Err("Okänd leverantör".to_string()),
    };

    // Bygg meddelanden
    let mut messages = Vec::new();
    
    for (is_user, text) in history {
        let role = if *is_user { "user" } else { "assistant" };
        
        // Om det är sista meddelandet (user) OCH vi har en bild -> Lägg till bild
        if *is_user && base64_image.is_some() && text == &history.last().unwrap().1 {
             let img_data = base64_image.as_ref().unwrap();
             messages.push(json!({
                "role": role,
                "content": [
                    { "type": "text", "text": text },
                    { 
                        "type": "image_url", 
                        "image_url": { "url": format!("data:image/png;base64,{}", img_data) } 
                    }
                ]
            }));
        } else {
            // Vanligt textmeddelande
            messages.push(json!({ "role": role, "content": text }));
        }
    }

    let body = json!({
        "model": model,
        "messages": messages
    });

    let res = client.post(url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .map_err(|e| format!("Nätverksfel: {}", e))?;

    if res.status().is_success() {
        let json: serde_json::Value = res.json().map_err(|e| e.to_string())?;
        // Plocka ut svaret: choices[0].message.content
        json["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or("Kunde inte läsa svaret från API".to_string())
    } else {
        let err_text = res.text().unwrap_or_default();
        Err(format!("API Fel: {}", err_text))
    }
}

// --- ANTHROPIC LOGIK ---
fn send_anthropic(
    client: Client,
    api_key: &str,
    model: &str,
    base64_image: Option<String>,
    history: &Vec<(bool, String)>
) -> Result<String, String> {
    
    let mut messages = Vec::new();

    for (is_user, text) in history {
        let role = if *is_user { "user" } else { "assistant" };

        if *is_user && base64_image.is_some() && text == &history.last().unwrap().1 {
            let img_data = base64_image.as_ref().unwrap();
            messages.push(json!({
                "role": role,
                "content": [
                    { 
                        "type": "image", 
                        "source": { 
                            "type": "base64", 
                            "media_type": "image/png", 
                            "data": img_data 
                        } 
                    },
                    { "type": "text", "text": text }
                ]
            }));
        } else {
            messages.push(json!({ "role": role, "content": text }));
        }
    }

    let body = json!({
        "model": model,
        "max_tokens": 1024,
        "messages": messages
    });

    let res = client.post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .map_err(|e| format!("Nätverksfel: {}", e))?;

    if res.status().is_success() {
        let json: serde_json::Value = res.json().map_err(|e| e.to_string())?;
        // Plocka ut svaret: content[0].text
        json["content"][0]["text"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or("Kunde inte läsa svaret från Anthropic".to_string())
    } else {
        let err_text = res.text().unwrap_or_default();
        Err(format!("API Fel: {}", err_text))
    }
}