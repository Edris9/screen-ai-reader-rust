use serde::{Deserialize, Serialize};
use reqwest::blocking::Client;
use std::time::Duration;
use image::DynamicImage;
use image::ImageFormat; // <--- ÄNDRING 1: Vi använder ImageFormat istället
use std::io::Cursor;
use base64::{Engine as _, engine::general_purpose};

// Strukturer för Ollamas API
#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<OllamaMessage>,
    stream: bool,
}

#[derive(Serialize)]
struct OllamaMessage {
    role: String,
    content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    images: Option<Vec<String>>,
}

#[derive(Deserialize)]
struct ChatResponse {
    message: OllamaResponseContent,
}

#[derive(Deserialize)]
struct OllamaResponseContent {
    content: String,
}

// Strukturer för Tags (Modell-listan)
#[derive(Deserialize, Debug)]
struct OllamaTagResponse {
    models: Vec<OllamaModel>,
}
#[derive(Deserialize, Debug)]
struct OllamaModel {
    name: String,
}

// Hämta modeller
pub fn fetch_models() -> Result<Vec<String>, String> {
    let client = Client::builder()
        .timeout(Duration::from_secs(2)) // Kort timeout ifall Ollama inte körs
        .build()
        .map_err(|e| e.to_string())?;

    let res = client
        .get("http://localhost:11434/api/tags")
        .send();

    match res {
        Ok(response) => {
            if response.status().is_success() {
                let parsed: OllamaTagResponse = response.json().map_err(|e| e.to_string())?;
                let names = parsed.models.into_iter().map(|m| m.name).collect();
                Ok(names)
            } else {
                Err(format!("Ollama svarade med felkod: {}", response.status()))
            }
        }
        Err(_) => {
            Err("Kunde inte ansluta till Ollama. Körs programmet?".to_string())
        }
    }
}

// Skicka chatt
pub fn send_chat(
    model: String, 
    prompt: String, 
    image: Option<&DynamicImage>, 
    history: &Vec<crate::app::ChatMessage>
) -> Result<String, String> {
    
    let client = Client::builder().timeout(Duration::from_secs(60)).build().map_err(|e| e.to_string())?;

    // 1. Bygg meddelande-listan
    let mut messages: Vec<OllamaMessage> = history.iter().map(|msg| {
        OllamaMessage {
            role: if msg.is_user { "user".to_string() } else { "assistant".to_string() },
            content: msg.text.clone(),
            images: None,
        }
    }).collect();

    // 2. Förbered bilden
    let images_list = if let Some(img) = image {
        let mut bytes: Vec<u8> = Vec::new();
        // ÄNDRING 2: Använd ImageFormat::Png här
        img.write_to(&mut Cursor::new(&mut bytes), ImageFormat::Png)
            .map_err(|_| "Kunde inte behandla bilden".to_string())?;
        
        let b64 = general_purpose::STANDARD.encode(&bytes);
        Some(vec![b64])
    } else {
        None
    };

    // 3. Koppla bilden till sista meddelandet
    if let Some(last) = messages.last_mut() {
        if last.role == "user" {
            last.images = images_list;
        }
    }

    let request = ChatRequest {
        model,
        messages,
        stream: false,
    };

    // 4. Skicka
    let res = client.post("http://localhost:11434/api/chat")
        .json(&request)
        .send();

    match res {
        Ok(response) => {
            if response.status().is_success() {
                let parsed: ChatResponse = response.json().map_err(|e| e.to_string())?;
                Ok(parsed.message.content)
            } else {
                Err(format!("Ollama fel: {}", response.status()))
            }
        }
        Err(e) => Err(format!("Kunde inte skicka: {}", e)),
    }
}