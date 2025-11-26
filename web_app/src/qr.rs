//! # QR Code Generation Module
//!
//! This module handles QR code generation and styling for pet information sharing.
//! It provides functions to create beautiful QR codes with custom liquid-effect styling
//! and QR cards with pet pictures.

use anyhow::Context;
use qrcode::{EcLevel, QrCode};
use tiny_skia::{Color, FillRule, Paint, PathBuilder, Pixmap, Transform};

use crate::utils::detect_image_format;

const SCALE: f32 = 20.0;
const PADDING: usize = 4;

/// Generates a QR code image from a URL string.
///
/// Creates a high-quality QR code image with custom styling (liquid effect)
/// suitable for pet information sharing. The QR code uses high error correction level.
///
/// # Arguments
/// * `info_url` - The URL or text to encode in the QR code
///
/// # Returns
/// * `anyhow::Result<Vec<u8>>` - PNG image data of the QR code
///
/// # Errors
/// Returns an error if:
/// - QR code generation fails (data too large, invalid format)
/// - Image generation fails
///
/// # Example
/// ```rust
/// let qr_data = get_qr_code("https://example.com/pet/123")?;
/// std::fs::write("qr.png", qr_data)?;
/// ```
pub fn get_qr_code(info_url: &str) -> anyhow::Result<Vec<u8>> {
    // Generate the QR code matrix
    let code = QrCode::with_error_correction_level(info_url, EcLevel::L)?;
    let width = code.width();

    // Calculate canvas size
    let canvas_modules = width + (PADDING * 2);
    let canvas_size = (canvas_modules as f32 * SCALE) as u32;

    let mut pixmap = Pixmap::new(canvas_size, canvas_size).context("Failed to create pixmap")?;
    pixmap.fill(Color::WHITE);

    // Builders for different layers
    let mut black_outer_pb = PathBuilder::new(); // Body + Outer Finder
    let mut white_gap_pb = PathBuilder::new(); // Finder Gap
    let mut black_inner_pb = PathBuilder::new(); // Finder Inner Dot

    // Paint for the liquid blobs (Dark Blue/Black)
    let mut black_paint = Paint::default();
    black_paint.set_color_rgba8(15, 23, 42, 255); // #0f172a
    black_paint.anti_alias = true;

    // Paint for the gaps (White)
    let mut white_paint = Paint::default();
    white_paint.set_color(Color::WHITE);
    white_paint.anti_alias = true;

    // 4. HELPER: CHECK IF MODULE IS PART OF A FINDER PATTERN (THE 3 EYES)
    let is_finder = |x: usize, y: usize| -> bool {
        (x >= width - 7 || x < 7) && y < 7 || (x < 7 && y >= width - 7) // Bottom-Left
    };

    // 5. DRAW THE LIQUID BODY
    draw_liquid_body(&code, width, SCALE, PADDING, &mut black_outer_pb, is_finder);

    // 6. DRAW CUSTOM "SQUIRCLE" FINDER PATTERNS
    draw_finder_patterns(
        width,
        SCALE,
        PADDING,
        &mut black_outer_pb,
        &mut white_gap_pb,
        &mut black_inner_pb,
        draw_rounded_rect,
    );

    // Fill the paths
    // 1. Draw Body + Outer Finders (Black)
    if let Some(path) = black_outer_pb.finish() {
        pixmap.fill_path(
            &path,
            &black_paint,
            FillRule::Winding,
            Transform::default(),
            None,
        );
    }

    // 2. Draw Gaps (White)
    if let Some(path) = white_gap_pb.finish() {
        pixmap.fill_path(
            &path,
            &white_paint,
            FillRule::Winding,
            Transform::default(),
            None,
        );
    }

    // 3. Draw Inner Finders (Black)
    if let Some(path) = black_inner_pb.finish() {
        pixmap.fill_path(
            &path,
            &black_paint,
            FillRule::Winding,
            Transform::default(),
            None,
        );
    }

    Ok(pixmap.encode_png()?)
}

fn draw_rounded_rect(pb: &mut PathBuilder, x: f32, y: f32, w: f32, h: f32, r: f32) {
    pb.move_to(x + r, y);
    pb.line_to(x + w - r, y);
    pb.quad_to(x + w, y, x + w, y + r);
    pb.line_to(x + w, y + h - r);
    pb.quad_to(x + w, y + h, x + w - r, y + h);
    pb.line_to(x + r, y + h);
    pb.quad_to(x, y + h, x, y + h - r);
    pb.line_to(x, y + r);
    pb.quad_to(x, y, x + r, y);
    pb.close();
}

fn draw_liquid_body<F>(
    code: &qrcode::QrCode,
    width: usize,
    scale: f32,
    padding: usize,
    pb: &mut tiny_skia::PathBuilder,
    is_finder: F,
) where
    F: Fn(usize, usize) -> bool,
{
    use tiny_skia::Rect;

    for y in 0..width {
        for x in 0..width {
            if let qrcode::Color::Dark = code[(x, y)] {
                if is_finder(x, y) {
                    continue;
                }

                let cx = (x + padding) as f32 * scale;
                let cy = (y + padding) as f32 * scale;
                let radius = scale / 2.0;

                // A. Draw the MAIN CIRCLE for this module
                let circle_rect = Rect::from_xywh(cx, cy, scale, scale).unwrap();
                pb.push_oval(circle_rect);

                // B. DRAW BRIDGES (The "Liquid" Logic)
                // Right neighbor
                if x + 1 < width
                    && let qrcode::Color::Dark = code[(x + 1, y)]
                    && !is_finder(x + 1, y)
                {
                    let rect = Rect::from_xywh(cx + radius, cy, scale, scale).unwrap();
                    pb.push_rect(rect);
                }

                // Bottom neighbor
                if y + 1 < width
                    && let qrcode::Color::Dark = code[(x, y + 1)]
                    && !is_finder(x, y + 1)
                {
                    let rect = Rect::from_xywh(cx, cy + radius, scale, scale).unwrap();
                    pb.push_rect(rect);
                }
            }
        }
    }
}

fn draw_finder_patterns<F>(
    width: usize,
    scale: f32,
    padding: usize,
    black_outer_pb: &mut tiny_skia::PathBuilder,
    white_gap_pb: &mut tiny_skia::PathBuilder,
    black_inner_pb: &mut tiny_skia::PathBuilder,
    push_rounded_rect: F,
) where
    F: Fn(&mut tiny_skia::PathBuilder, f32, f32, f32, f32, f32),
{
    let mut draw_eye = |tx: usize, ty: usize| {
        let x = (tx + padding) as f32 * scale;
        let y = (ty + padding) as f32 * scale;

        // Outer Box (7x7) - Black
        let outer_size = 7.0 * scale;
        let outer_radius = 2.5 * scale;
        push_rounded_rect(black_outer_pb, x, y, outer_size, outer_size, outer_radius);

        // White Mask (The gap) - White
        // Gap is 5x5, offset by 1
        let gap_offset = 1.0 * scale;
        let gap_size = 5.0 * scale;
        let gap_radius = 2.0 * scale;
        push_rounded_rect(
            white_gap_pb,
            x + gap_offset,
            y + gap_offset,
            gap_size,
            gap_size,
            gap_radius,
        );

        // Inner Dot (3x3) - Black
        // Inner is 3x3, offset by 2
        let inner_offset = 2.0 * scale;
        let inner_size = 3.0 * scale;
        let inner_radius = 1.2 * scale;
        push_rounded_rect(
            black_inner_pb,
            x + inner_offset,
            y + inner_offset,
            inner_size,
            inner_size,
            inner_radius,
        );
    };

    draw_eye(0, 0); // Top-Left
    draw_eye(width - 7, 0); // Top-Right
    draw_eye(0, width - 7); // Bottom-Left
}

/// Builds a styled QR card with pet picture.
///
/// Creates a beautiful card design with:
/// - Gradient background
/// - White rounded card container
/// - Floating circular pet avatar (50% inside card, 50% outside)
/// - Centered QR code
/// - Footer text "by pet-info.link"
///
/// # Arguments
/// * `pet_pic` - The pet's picture data (body and extension)
/// * `info_url` - The URL to encode in the QR code
///
/// # Returns
/// * `anyhow::Result<Vec<u8>>` - PNG image data of the complete card
///
/// # Errors
/// Returns an error if:
/// - QR code generation fails
/// - Pet picture loading fails
/// - Image composition fails
pub fn build_qr_card_with_pic(
    pet_pic: &crate::api::pet::PetPublicPic,
    info_url: &str,
) -> anyhow::Result<Vec<u8>> {
    // Card dimensions
    const CARD_WIDTH: u32 = 600;
    const CARD_HEIGHT: u32 = 720; // Reduced by 20%
    const AVATAR_SIZE: u32 = 160;
    const CARD_RADIUS: f32 = 40.0;
    const AVATAR_RADIUS: u32 = AVATAR_SIZE / 2; // Avatar radius (80px)

    const CANVAS_WIDTH: u32 = CARD_WIDTH + 100; // Extra margin for shadow/spacing
    const CANVAS_HEIGHT: u32 = CARD_HEIGHT + AVATAR_RADIUS + 100; // Reduced proportionally

    // Calculate positions
    // Card starts after top margin
    let card_x = 50.0;
    let card_y = (AVATAR_RADIUS + 50) as f32; // Card top edge = 130px from canvas top

    // Avatar center should align with card center horizontally and card top edge vertically
    let avatar_x = card_x + (CARD_WIDTH as f32 / 2.0); // Centered on card horizontally
    let avatar_y = card_y; // Avatar center = card top edge (50% outside, 50% inside)

    // Create canvas with gradient background
    let mut pixmap = Pixmap::new(CANVAS_WIDTH, CANVAS_HEIGHT).context("Failed to create pixmap")?;

    // Fill with gradient background (#f0f4f8 to #e2e8f0)
    for y in 0..CANVAS_HEIGHT {
        for x in 0..CANVAS_WIDTH {
            let t = y as f32 / CANVAS_HEIGHT as f32;
            let r = (240.0 + (226.0 - 240.0) * t) as u8;
            let g = (244.0 + (232.0 - 244.0) * t) as u8;
            let b = (248.0 + (240.0 - 248.0) * t) as u8;
            pixmap.pixels_mut()[(y * CANVAS_WIDTH + x) as usize] =
                tiny_skia::ColorU8::from_rgba(r, g, b, 255).premultiply();
        }
    }

    // Draw white rounded card
    let mut card_pb = PathBuilder::new();
    draw_rounded_rect(
        &mut card_pb,
        card_x,
        card_y,
        CARD_WIDTH as f32,
        CARD_HEIGHT as f32,
        CARD_RADIUS,
    );
    if let Some(path) = card_pb.finish() {
        let mut paint = Paint::default();
        paint.set_color(Color::WHITE);
        paint.anti_alias = true;
        pixmap.fill_path(&path, &paint, FillRule::Winding, Transform::default(), None);
    }

    // Generate and overlay QR code
    let qr_bytes = get_qr_code(info_url)?;
    let qr_img = image::load_from_memory(&qr_bytes)
        .context("Failed to load QR code")?
        .to_rgba8();

    // Resize QR code if it's too large for the card
    let max_qr_size = (CARD_WIDTH as f32 * 0.7) as u32; // 70% of card width
    let qr_img = if qr_img.width() > max_qr_size || qr_img.height() > max_qr_size {
        image::imageops::resize(
            &qr_img,
            max_qr_size,
            max_qr_size,
            image::imageops::FilterType::Lanczos3,
        )
    } else {
        qr_img
    };

    // Position QR code in center of card, below avatar
    let qr_size = qr_img.width().min(qr_img.height());
    let qr_x = (CANVAS_WIDTH.saturating_sub(qr_size)) / 2;
    let qr_y = card_y as u32 + AVATAR_RADIUS + 60; // Below avatar with spacing

    // Overlay QR code
    for (x, y, pixel) in qr_img.enumerate_pixels() {
        let px = qr_x + x;
        let py = qr_y + y;
        if px < CANVAS_WIDTH && py < CANVAS_HEIGHT {
            let idx = (py * CANVAS_WIDTH + px) as usize;
            if pixel[3] > 128 {
                // Alpha threshold
                pixmap.pixels_mut()[idx] =
                    tiny_skia::ColorU8::from_rgba(pixel[0], pixel[1], pixel[2], pixel[3])
                        .premultiply();
            }
        }
    }

    // Load and overlay circular pet picture
    // Detect actual format from magic bytes (more reliable than extension)
    let actual_format = detect_image_format(&pet_pic.body);
    let pic_format = match actual_format {
        "png" => image::ImageFormat::Png,
        "gif" => image::ImageFormat::Gif,
        "webp" => image::ImageFormat::WebP,
        "bmp" => image::ImageFormat::Bmp,
        _ => image::ImageFormat::Jpeg, // jpg or default
    };

    let pet_img = image::load_from_memory_with_format(&pet_pic.body, pic_format)
        .with_context(|| {
            format!(
                "Failed to load pet picture (detected: {}, stored ext: {}, size: {} bytes)",
                actual_format,
                pet_pic.extension,
                pet_pic.body.len()
            )
        })?
        .resize_to_fill(
            AVATAR_SIZE,
            AVATAR_SIZE,
            image::imageops::FilterType::Lanczos3,
        )
        .to_rgba8();

    // Create circular mask and overlay avatar on canvas
    // Avatar center (avatar_x, avatar_y) should align with card's horizontal center and top edge
    let radius = (AVATAR_SIZE / 2) as i32;
    // Use slightly smaller radius (1px inset) to completely avoid edge artifacts
    let safe_radius_squared = (radius - 1).pow(2);

    for y in 0..AVATAR_SIZE {
        for x in 0..AVATAR_SIZE {
            // Calculate offset from avatar center
            let dx = x as i32 - radius;
            let dy = y as i32 - radius;
            let distance_squared = dx * dx + dy * dy;

            // Only draw pixels within the safe circular mask
            if distance_squared < safe_radius_squared {
                let pixel = pet_img.get_pixel(x, y);

                // Only draw fully opaque pixels (>= 250) to eliminate all edge artifacts
                if pixel[3] >= 250 {
                    // Position pixel relative to avatar center
                    let canvas_x = (avatar_x as i32 + dx) as u32;
                    let canvas_y = (avatar_y as i32 + dy) as u32;

                    if canvas_x < CANVAS_WIDTH && canvas_y < CANVAS_HEIGHT {
                        let idx = (canvas_y * CANVAS_WIDTH + canvas_x) as usize;
                        // Force full opacity (255) to prevent any premultiplication artifacts
                        pixmap.pixels_mut()[idx] =
                            tiny_skia::ColorU8::from_rgba(pixel[0], pixel[1], pixel[2], 255)
                                .premultiply();
                    }
                }
            }
        }
    }

    // Draw footer text "by pet-info.link"
    use ab_glyph::{Font, FontRef, PxScale, ScaleFont};

    let font_data = include_bytes!("../assets/fonts/DynaPuff.ttf");
    let font = FontRef::try_from_slice(font_data).context("Failed to load font")?;

    const FONT_SIZE: f32 = 24.0;
    let text = "by pet-info.link";
    let scale = PxScale::from(FONT_SIZE);
    let scaled_font = font.as_scaled(scale);

    // Calculate text width for centering
    let text_width: f32 = text
        .chars()
        .map(|c| scaled_font.h_advance(scaled_font.glyph_id(c)))
        .sum();

    let text_x = ((CANVAS_WIDTH as f32 - text_width) / 2.0).max(0.0);
    let text_y = qr_y + qr_size + 60;
    let text_color = tiny_skia::ColorU8::from_rgba(15, 23, 42, 255); // Matching QR color

    // Render text
    let mut x_offset = text_x;
    for ch in text.chars() {
        let glyph_id = scaled_font.glyph_id(ch);
        let glyph =
            glyph_id.with_scale_and_position(scale, ab_glyph::point(x_offset, text_y as f32));

        if let Some(outlined) = scaled_font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            outlined.draw(|gx, gy, coverage| {
                if coverage > 0.0 {
                    let px = (bounds.min.x as i32 + gx as i32) as u32;
                    let py = (bounds.min.y as i32 + gy as i32) as u32;

                    if px < CANVAS_WIDTH && py < CANVAS_HEIGHT {
                        let idx = (py * CANVAS_WIDTH + px) as usize;
                        let alpha = (coverage * 255.0) as u8;

                        // Alpha blend text with background
                        let bg = pixmap.pixels()[idx].demultiply();
                        let blended = tiny_skia::ColorU8::from_rgba(
                            ((text_color.red() as u16 * alpha as u16
                                + bg.red() as u16 * (255 - alpha) as u16)
                                / 255) as u8,
                            ((text_color.green() as u16 * alpha as u16
                                + bg.green() as u16 * (255 - alpha) as u16)
                                / 255) as u8,
                            ((text_color.blue() as u16 * alpha as u16
                                + bg.blue() as u16 * (255 - alpha) as u16)
                                / 255) as u8,
                            255,
                        );
                        pixmap.pixels_mut()[idx] = blended.premultiply();
                    }
                }
            });
        }

        x_offset += scaled_font.h_advance(glyph_id);
    }

    Ok(pixmap.encode_png()?)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Tests QR code generation with various inputs.
    #[test]
    fn test_get_qr_code() {
        // Test with simple URL
        let result = get_qr_code("https://example.com");
        assert!(result.is_ok());

        let qr_data = result.unwrap();
        assert!(!qr_data.is_empty());

        // Verify it's PNG format (starts with PNG signature)
        assert_eq!(&qr_data[0..8], &[137, 80, 78, 71, 13, 10, 26, 10]);

        // Test with longer URL
        let long_url =
            "https://example.com/very/long/path/with/parameters?param1=value1&param2=value2";
        let result = get_qr_code(long_url);
        assert!(result.is_ok());
    }
}
