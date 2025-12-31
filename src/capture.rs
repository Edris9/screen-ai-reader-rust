use image::{DynamicImage, ImageBuffer, Rgba};
use screenshots::Screen;
use std::io::Cursor;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

#[derive(Clone, Debug)]
pub struct CaptureRegion {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

pub struct ScreenCapture {
    screens: Vec<Screen>,
}

impl ScreenCapture {
    pub fn new() -> Result<Self, String> {
        let screens = Screen::all().map_err(|e| format!("Kunde inte hämta skärmar: {}", e))?;
        Ok(Self { screens })
    }

    /// Fångar hela primära skärmen
    pub fn capture_full(&self) -> Result<DynamicImage, String> {
        let screen = self.screens.first().ok_or("Ingen skärm hittades")?;
        let capture = screen.capture().map_err(|e| format!("Fångst misslyckades: {}", e))?;
        
        let img = ImageBuffer::<Rgba<u8>, _>::from_raw(
            capture.width(),
            capture.height(),
            capture.to_vec(),
        ).ok_or("Kunde inte skapa bild")?;
        
        Ok(DynamicImage::ImageRgba8(img))
    }

    /// Fångar en specifik region
    pub fn capture_region(&self, region: &CaptureRegion) -> Result<DynamicImage, String> {
        let screen = self.screens.first().ok_or("Ingen skärm hittades")?;
        
        let capture = screen.capture_area(
            region.x,
            region.y,
            region.width,
            region.height,
        ).map_err(|e| format!("Region-fångst misslyckades: {}", e))?;
        
        let img = ImageBuffer::<Rgba<u8>, _>::from_raw(
            capture.width(),
            capture.height(),
            capture.to_vec(),
        ).ok_or("Kunde inte skapa bild")?;
        
        Ok(DynamicImage::ImageRgba8(img))
    }

    /// Konverterar bild till base64 PNG
    pub fn image_to_base64(img: &DynamicImage) -> Result<String, String> {
        let mut buffer = Cursor::new(Vec::new());
        img.write_to(&mut buffer, image::ImageFormat::Png)
            .map_err(|e| format!("Kunde inte koda bild: {}", e))?;
        
        Ok(BASE64.encode(buffer.into_inner()))
    }

    /// Hämtar skärmstorlek
    pub fn get_screen_size(&self) -> Option<(u32, u32)> {
        self.screens.first().map(|s| {
            (s.display_info.width, s.display_info.height)
        })
    }
}
