use image::DynamicImage;
use screenshots::Screen;

pub fn take_screenshot_delayed() -> Option<DynamicImage> {
    // Vänta så fönstret hinner gömmas
    std::thread::sleep(std::time::Duration::from_millis(300));
    take_screenshot()
}

pub fn take_screenshot() -> Option<DynamicImage> {
    let screens = Screen::all().ok()?;
    let screen = screens.first()?;
    let capture = screen.capture().ok()?;
    
    let img = DynamicImage::ImageRgba8(
        image::ImageBuffer::from_raw(
            capture.width(),
            capture.height(),
            capture.to_vec(),
        )?
    );
    
    println!("Screenshot tagen! {}x{}", capture.width(), capture.height());
    Some(img)
}