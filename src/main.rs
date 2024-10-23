use actix_web::{
    get, middleware::DefaultHeaders, web, App, HttpRequest, HttpResponse, HttpServer, Responder,
};
use chrono::Utc;
use image::{ImageBuffer, Rgba};
use rusttype::{Font, Scale};
use serde::Deserialize; // Import Deserialize derive macro
use std::{env, io::Cursor};

#[derive(Deserialize)]
struct ImageOptions {
    width: Option<u32>,
}

#[get("/")]
async fn index(query: web::Query<ImageOptions>) -> impl Responder {
    // Get current Unix epoch time
    let epoch_time = Utc::now().timestamp();

    // Get width from query parameter or default
    let width = query.width.unwrap_or(320);

    // Validate width
    let width = if width >= 10 && width <= 10000 {
        width
    } else {
        320
    };

    // Generate the image
    let img = generate_image(epoch_time.to_string(), width);

    // Convert image to PNG bytes
    let mut img_buffer = Cursor::new(Vec::new());
    img.write_to(&mut img_buffer, image::ImageFormat::Png)
        .unwrap();

    // Return the image as HTTP response with Cache-Control header
    HttpResponse::Ok()
        .content_type("image/png")
        .body(img_buffer.into_inner())
}

#[get("/{epoch}.{ext}")]
async fn image_with_epoch(
    path: web::Path<(String, String)>,
    query: web::Query<ImageOptions>,
    req: HttpRequest,
) -> impl Responder {
    let (epoch_str, ext) = path.into_inner();

    if epoch_str == "epoch" || epoch_str == "latest" {
        // Handle redirect
        let epoch_time = Utc::now().timestamp().to_string();
        // Build new URL with the current epoch time
        let mut new_path = format!("/{epoch}.{ext}", epoch = epoch_time, ext = ext);

        // Get the query parameters from the request
        let query_string = req.query_string();
        if !query_string.is_empty() {
            new_path = format!("{}?{}", new_path, query_string);
        }

        // Redirect to the new URL
        return HttpResponse::TemporaryRedirect()
            .append_header(("Location", new_path))
            .finish();
    }

    // Try to parse epoch_str to an integer
    let epoch_time = epoch_str
        .parse::<i64>()
        .unwrap_or_else(|_| Utc::now().timestamp());

    // Get width from query parameter or default
    let width = query.width.unwrap_or(320);

    // Validate width
    let width = if width >= 10 && width <= 10000 {
        width
    } else {
        320
    };

    // Generate the image
    let img = generate_image(epoch_time.to_string(), width);

    // Convert image to the desired format based on ext
    let mut img_buffer = Cursor::new(Vec::new());

    let format = match ext.as_str() {
        "png" => image::ImageFormat::Png,
        "jpg" | "jpeg" => image::ImageFormat::Jpeg,
        _ => image::ImageFormat::Png, // default to PNG
    };

    img.write_to(&mut img_buffer, format).unwrap();

    // Set content type based on format
    let content_type = match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        _ => "application/octet-stream",
    };

    // Return the image as HTTP response with Cache-Control header
    HttpResponse::Ok()
        .content_type(content_type)
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
    HttpServer::new(|| {
        App::new()
            .wrap(DefaultHeaders::new().add(("Cache-Control", "max-age=2592000")))
            .service(index)
            .service(image_with_epoch)
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}

fn generate_image(text: String, width: u32) -> ImageBuffer<Rgba<u8>, Vec<u8>> {
    // Ensure width is valid, or set default
    let width = if width >= 10 && width <= 10000 { width } else { 320 };
    let height = width / 4;

    // Create a new image buffer with a white background
    let mut img = ImageBuffer::from_pixel(width, height, Rgba([255, 255, 255, 255]));

    // Load the font
    let font_data = include_bytes!("../fonts/Courier_New.ttf") as &[u8];
    let font = Font::try_from_bytes(font_data).unwrap();

    // Define the scale of the font based on image size
    let scale = Scale::uniform(height as f32 * 0.8);

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
        let positioned_glyph = glyph
            .clone()
            .into_unpositioned()
            .positioned(rusttype::point(
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
                        ((fg[0] as u16 * alpha as u16 + bg[0] as u16 * inv_alpha as u16) / 255)
                            as u8,
                        ((fg[1] as u16 * alpha as u16 + bg[1] as u16 * inv_alpha as u16) / 255)
                            as u8,
                        ((fg[2] as u16 * alpha as u16 + bg[2] as u16 * inv_alpha as u16) / 255)
                            as u8,
                        255,
                    ];
                }
            });
        }
    }

    img
}
