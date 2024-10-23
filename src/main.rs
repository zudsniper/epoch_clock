use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use chrono::Utc;
use image::{ImageBuffer, Rgba};
use rusttype::{Font, Scale};
use std::io::Cursor;

#[get("/")]
async fn index() -> impl Responder {
    // Get current Unix epoch time
    let epoch_time = Utc::now().timestamp();

    // Generate the image
    let img = generate_image(epoch_time.to_string());

    // Convert image to PNG bytes
    let mut img_buffer = Cursor::new(Vec::new());
    img.write_to(&mut img_buffer, image::ImageOutputFormat::Png)
        .unwrap();

    // Return the image as HTTP response
    HttpResponse::Ok()
        .content_type("image/png")
        .body(img_buffer.into_inner())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Server running at http://localhost:8055/");
    HttpServer::new(|| App::new().service(index))
        .bind("127.0.0.1:8055")?
        .run()
        .await
}

fn generate_image(text: String) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    // Image dimensions
    let width = 400;
    let height = 100;

    // Create a new image buffer with a background color
    let mut img = ImageBuffer::from_pixel(width, height, Rgba([255, 255, 255, 255]));

    // Load a font
    let font_data = include_bytes!("../fonts/Courier_New.ttf") as &[u8];
    let font = Font::try_from_bytes(font_data).unwrap();

    // Define the scale of the font
    let scale = Scale {
        x: 50.0,
        y: 50.0,
    };

    // Calculate text position for centering
    let v_metrics = font.v_metrics(scale);
    let glyphs: Vec<_> = font.layout(&text, scale, rusttype::point(0.0, 0.0 + v_metrics.ascent)).collect();
    let width_offset = (width as f32 - glyphs.iter().rev().next().unwrap().position().x) / 2.0;
    let height_offset = (height as f32 - (v_metrics.ascent - v_metrics.descent)) / 2.0 + v_metrics.ascent;

    // Draw the text onto the image
    for glyph in glyphs {
        if let Some(bounding_box) = glyph.pixel_bounding_box() {
            glyph.draw(|x, y, v| {
                let x = x + bounding_box.min.x as u32 + width_offset as u32;
                let y = y + bounding_box.min.y as u32 + height_offset as u32;
                if x < width && y < height {
                    let pixel = img.get_pixel_mut(x, y);
                    *pixel = Rgba([0, 0, 0, (v * 255.0) as u8]);
                }
            });
        }
    }

    img
}
