use serde::Deserialize;
use reqwest::blocking::Client;
use std::time::Duration;

#[derive(Deserialize, Debug)]
struct OllamaTagResponse {
    models: Vec<OllamaModel>,
}

#[derive(Deserialize, Debug)]
struct OllamaModel {
    name: String,
}

// Funktion för att hämta modeller
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