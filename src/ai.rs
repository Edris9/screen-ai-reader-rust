use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use futures_util::StreamExt;

#[derive(Clone, Debug)]
pub enum ModelProvider {
    Local { endpoint: String, model: String },
    OpenAI { api_key: String, model: String },
    Claude { api_key: String, model: String },
}

impl Default for ModelProvider {
    fn default() -> Self {
        // Standard: Ollama lokal
        Self::Local {
            endpoint: "http://localhost:11434".to_string(),
            model: "llava".to_string(),
        }
    }
}

#[derive(Clone)]
pub struct AiClient {
    client: Client,
    pub provider: ModelProvider,
}

// Ollama structs
#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    images: Vec<String>,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
    done: bool,
}

// OpenAI structs
#[derive(Serialize)]
struct OpenAIRequest {
    model: String,
    messages: Vec<OpenAIMessage>,
    stream: bool,
    max_tokens: u32,
}

#[derive(Serialize)]
struct OpenAIMessage {
    role: String,
    content: Vec<OpenAIContent>,
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum OpenAIContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image_url")]
    ImageUrl { image_url: ImageUrl },
}

#[derive(Serialize)]
struct ImageUrl {
    url: String,
}

#[derive(Deserialize)]
struct OpenAIStreamResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Deserialize)]
struct OpenAIChoice {
    delta: OpenAIDelta,
}

#[derive(Deserialize)]
struct OpenAIDelta {
    content: Option<String>,
}

// Claude structs
#[derive(Serialize)]
struct ClaudeRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<ClaudeMessage>,
    stream: bool,
}

#[derive(Serialize)]
struct ClaudeMessage {
    role: String,
    content: Vec<ClaudeContent>,
}

#[derive(Serialize)]
#[serde(tag = "type")]
enum ClaudeContent {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "image")]
    Image { source: ClaudeImageSource },
}

#[derive(Serialize)]
struct ClaudeImageSource {
    #[serde(rename = "type")]
    source_type: String,
    media_type: String,
    data: String,
}

impl AiClient {
    pub fn new(provider: ModelProvider) -> Self {
        Self {
            client: Client::new(),
            provider,
        }
    }

    pub fn set_provider(&mut self, provider: ModelProvider) {
        self.provider = provider;
    }

    /// Skickar bild till AI och streamar svaret
    pub async fn analyze_image(
        &self,
        image_base64: String,
        prompt: String,
        tx: mpsc::UnboundedSender<String>,
        cancel_rx: &mut mpsc::Receiver<()>,
    ) -> Result<(), String> {
        match &self.provider {
            ModelProvider::Local { endpoint, model } => {
                self.call_ollama(endpoint, model, image_base64, prompt, tx, cancel_rx).await
            }
            ModelProvider::OpenAI { api_key, model } => {
                self.call_openai(api_key, model, image_base64, prompt, tx, cancel_rx).await
            }
            ModelProvider::Claude { api_key, model } => {
                self.call_claude(api_key, model, image_base64, prompt, tx, cancel_rx).await
            }
        }
    }

    async fn call_ollama(
        &self,
        endpoint: &str,
        model: &str,
        image_base64: String,
        prompt: String,
        tx: mpsc::UnboundedSender<String>,
        cancel_rx: &mut mpsc::Receiver<()>,
    ) -> Result<(), String> {
        let request = OllamaRequest {
            model: model.to_string(),
            prompt,
            images: vec![image_base64],
            stream: true,
        };

        let response = self.client
            .post(format!("{}/api/generate", endpoint))
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Ollama request failed: {}", e))?;

        let mut stream = response.bytes_stream();
        
        loop {
            tokio::select! {
                _ = cancel_rx.recv() => {
                    return Ok(());
                }
                chunk = stream.next() => {
                    match chunk {
                        Some(Ok(bytes)) => {
                            if let Ok(text) = std::str::from_utf8(&bytes) {
                                for line in text.lines() {
                                    if let Ok(resp) = serde_json::from_str::<OllamaResponse>(line) {
                                        let _ = tx.send(resp.response);
                                        if resp.done {
                                            return Ok(());
                                        }
                                    }
                                }
                            }
                        }
                        Some(Err(e)) => return Err(format!("Stream error: {}", e)),
                        None => return Ok(()),
                    }
                }
            }
        }
    }

    async fn call_openai(
        &self,
        api_key: &str,
        model: &str,
        image_base64: String,
        prompt: String,
        tx: mpsc::UnboundedSender<String>,
        cancel_rx: &mut mpsc::Receiver<()>,
    ) -> Result<(), String> {
        let request = OpenAIRequest {
            model: model.to_string(),
            messages: vec![OpenAIMessage {
                role: "user".to_string(),
                content: vec![
                    OpenAIContent::Text { text: prompt },
                    OpenAIContent::ImageUrl {
                        image_url: ImageUrl {
                            url: format!("data:image/png;base64,{}", image_base64),
                        },
                    },
                ],
            }],
            stream: true,
            max_tokens: 4096,
        };

        let response = self.client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("OpenAI request failed: {}", e))?;

        let mut stream = response.bytes_stream();
        
        loop {
            tokio::select! {
                _ = cancel_rx.recv() => {
                    return Ok(());
                }
                chunk = stream.next() => {
                    match chunk {
                        Some(Ok(bytes)) => {
                            if let Ok(text) = std::str::from_utf8(&bytes) {
                                for line in text.lines() {
                                    if line.starts_with("data: ") && line != "data: [DONE]" {
                                        let json_str = &line[6..];
                                        if let Ok(resp) = serde_json::from_str::<OpenAIStreamResponse>(json_str) {
                                            if let Some(content) = resp.choices.first()
                                                .and_then(|c| c.delta.content.clone()) {
                                                let _ = tx.send(content);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Some(Err(e)) => return Err(format!("Stream error: {}", e)),
                        None => return Ok(()),
                    }
                }
            }
        }
    }

    async fn call_claude(
        &self,
        api_key: &str,
        model: &str,
        image_base64: String,
        prompt: String,
        tx: mpsc::UnboundedSender<String>,
        cancel_rx: &mut mpsc::Receiver<()>,
    ) -> Result<(), String> {
        let request = ClaudeRequest {
            model: model.to_string(),
            max_tokens: 4096,
            messages: vec![ClaudeMessage {
                role: "user".to_string(),
                content: vec![
                    ClaudeContent::Image {
                        source: ClaudeImageSource {
                            source_type: "base64".to_string(),
                            media_type: "image/png".to_string(),
                            data: image_base64,
                        },
                    },
                    ClaudeContent::Text { text: prompt },
                ],
            }],
            stream: true,
        };

        let response = self.client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| format!("Claude request failed: {}", e))?;

        let mut stream = response.bytes_stream();
        
        loop {
            tokio::select! {
                _ = cancel_rx.recv() => {
                    return Ok(());
                }
                chunk = stream.next() => {
                    match chunk {
                        Some(Ok(bytes)) => {
                            if let Ok(text) = std::str::from_utf8(&bytes) {
                                for line in text.lines() {
                                    if line.starts_with("data: ") {
                                        let json_str = &line[6..];
                                        if let Ok(event) = serde_json::from_str::<serde_json::Value>(json_str) {
                                            if event["type"] == "content_block_delta" {
                                                if let Some(text) = event["delta"]["text"].as_str() {
                                                    let _ = tx.send(text.to_string());
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        Some(Err(e)) => return Err(format!("Stream error: {}", e)),
                        None => return Ok(()),
                    }
                }
            }
        }
    }
}
