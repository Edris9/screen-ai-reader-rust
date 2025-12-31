# 🔮 Screen AI Reader

En snabb Rust-app för att fånga delar av skärmen och analysera med AI.

## Funktioner

- 📸 **Skärmfångst** - Dra en rektangel för att välja område
- 🖥️ **Lokal modell** - Stöd för Ollama (LLaVA, Llama-vision, etc.)
- ☁️ **Online modeller** - OpenAI GPT-4o och Claude stöd
- 📜 **Historik** - Spara tidigare analyser
- ⚡ **Streaming** - Se svaret medan det genereras
- 🎨 **Lila tema** - Snygg modern design

## Installation

### Förutsättningar

1. **Rust** - Installera från https://rustup.rs/
2. **Lokal modell** (valfritt) - Installera Ollama:
   ```bash
   curl -fsSL https://ollama.com/install.sh | sh
   ollama pull llava
   ```

### Bygg och kör

```bash
# Klona/kopiera projektet
cd screen-ai-reader

# Bygg (release för snabbhet)
cargo build --release

# Kör
cargo run --release
```

## Användning

1. **Starta appen** - Ett lila fönster öppnas
2. **Klicka "📸 Ny Skärmdump"** - Skärmen mörknar
3. **Dra en rektangel** - Markera området du vill analysera
4. **Skriv prompt** (valfritt) - Berätta vad AI:n ska göra
5. **Klicka "🚀 Analysera"** - Vänta på svaret

### Tangentbord

- `ESC` - Avbryt skärmfångst

## Konfiguration

Klicka på ⚙️ för att:
- Ställa in Ollama endpoint och modell
- Lägga till OpenAI API-nyckel
- Lägga till Claude API-nyckel
- Ändra standard-prompt

Config sparas i:
- **Linux/Mac**: `~/.config/screen-ai-reader/config.json`
- **Windows**: `%APPDATA%\screen-ai-reader\config.json`

## Modeller

### Lokal (Ollama)
```bash
# Vision-modeller som fungerar:
ollama pull llava          # Bäst balans
ollama pull llava:34b      # Mer kapabel
ollama pull bakllava       # Alternativ
```

### Online
- **OpenAI**: `gpt-4o` (rekommenderad), `gpt-4-vision-preview`
- **Claude**: `claude-sonnet-4-20250514` (snabb), `claude-opus-4-20250514` (smartast)

## Beroenden

- `eframe/egui` - GUI
- `screenshots` - Skärmfångst
- `reqwest` - HTTP-requests
- `tokio` - Async runtime
- `serde` - Serialisering

## Tips för snabbhet

1. **Använd release-build**: `cargo run --release`
2. **Håll Ollama igång**: Första requesten laddar modellen
3. **Mindre modeller är snabbare**: `llava` istället för `llava:34b`
4. **Streaming**: Svaret börjar visas direkt

## Licens

MIT
