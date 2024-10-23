use actix_web::{get, App, HttpResponse, HttpServer, Responder};
use chrono::Utc;
use image::{ImageBuffer, Rgba};
use rusttype::{Font, Scale};
use std::{env, io::Cursor};

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
    println!("Server starting...");
    let app_env = env::var("APP_ENV").unwrap_or_else(|_| "production".to_string());
    let host = if app_env == "dev" || app_env == "development" {
        env::var("HOST").unwrap_or_else(|_| "localhost".to_string())
    } else {
        env::var("HOST").expect("Host not set")
    };
    let port = if app_env == "dev" || app_env == "development" {
        env::var("PORT").unwrap_or_else(|_| "8055".to_string())
    } else {
        env::var("PORT").expect("Port not set")
    };
    HttpServer::new(|| App::new().service(index))
        .bind(format!("{}:{}", host, port))?
        .run()
        .await
}

fn generate_image(text: String) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    // Image dimensions
    let width = 320;
    let height = 80;

    // Create a new image buffer with a white background
    let mut img = ImageBuffer::from_pixel(width, height, Rgba([255, 255, 255, 255]));

    // Load the font
    let font_data = include_bytes!("../fonts/Courier_New.ttf") as &[u8];
    let font = Font::try_from_bytes(font_data).unwrap();

    // Define the scale of the font
    let scale = Scale { x: 50.0, y: 50.0 };

    // Layout the glyphs
    let glyphs: Vec<_> = font
        .layout(&text, scale, rusttype::point(0.0, 0.0))
        .collect();

    // Calculate the bounding box of the glyphs
    let mut min_x = std::f32::MAX;
    let mut max_x = std::f32::MIN;
    let mut min_y = std::f32::MAX;
    let mut max_y = std::f32::MIN;

    for glyph in &glyphs {
        if let Some(bb) = glyph.pixel_bounding_box() {
            if (bb.min.x as f32) < min_x {
                min_x = bb.min.x as f32;
            }
            if (bb.max.x as f32) > max_x {
                max_x = bb.max.x as f32;
            }
            if (bb.min.y as f32) < min_y {
                min_y = bb.min.y as f32;
            }
            if (bb.max.y as f32) > max_y {
                max_y = bb.max.y as f32;
            }
        }
    }

    // Handle the case where there are no glyphs or bounding boxes
    if min_x == std::f32::MAX {
        min_x = 0.0;
        max_x = 0.0;
    }
    if min_y == std::f32::MAX {
        min_y = 0.0;
        max_y = 0.0;
    }

    let text_width = max_x - min_x;
    let text_height = max_y - min_y;

    // Calculate offsets to center the text
    let x_offset = (width as f32 - text_width) / 2.0 - min_x;
    let y_offset = (height as f32 - text_height) / 2.0 - min_y;

    // Draw the glyphs onto the image
    for glyph in &glyphs {
        let positioned_glyph = glyph.clone().into_unpositioned().positioned(rusttype::point(
            glyph.position().x + x_offset,
            glyph.position().y + y_offset,
        ));

        if let Some(bb) = positioned_glyph.pixel_bounding_box() {
            positioned_glyph.draw(|x, y, v| {
                let px = x + bb.min.x as u32;
                let py = y + bb.min.y as u32;

                if px < width && py < height {
                    let pixel = img.get_pixel_mut(px, py);

                    // Blend the pixel values
                    let alpha = (v * 255.0) as u8;
                    let fg = [0u8, 0u8, 0u8, alpha]; // Black color with variable alpha

                    let bg = pixel.0;
                    let inv_alpha = 255 - alpha;
                    pixel.0 = [
                        ((fg[0] as u16 * alpha as u16 + bg[0] as u16 * inv_alpha as u16) / 255) as u8,
                        ((fg[1] as u16 * alpha as u16 + bg[1] as u16 * inv_alpha as u16) / 255) as u8,
                        ((fg[2] as u16 * alpha as u16 + bg[2] as u16 * inv_alpha as u16) / 255) as u8,
                        255,
                    ];
                }
            });
        }
    }

    img
}


