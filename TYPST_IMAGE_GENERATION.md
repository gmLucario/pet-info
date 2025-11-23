# Typst-based Image Generation

This document explains how to generate images from HTML templates using Typst in the pet-info application.

## Overview

We've implemented a system to convert pet information into shareable PNG images using the Typst typesetting system. This is useful for:
- Social media sharing
- WhatsApp/Telegram sharing
- Generating pet profile cards
- Creating printable pet info cards

## How It Works

1. **Typst Template**: Instead of HTML, we use Typst markup (`.typ` files) which is a modern typesetting language
2. **Data Binding**: We use Tera templating within Typst files to inject dynamic data
3. **Rendering**: Typst compiles the template and renders it directly to PNG format

## Architecture

```
Pet Data (Rust struct)
    ↓
Tera Template Rendering (inject data into .typ file)
    ↓
Typst Compilation (parse .typ markup)
    ↓
typst-render (convert to PNG bitmap)
    ↓
PNG Image Bytes
```

## Files Created

### 1. Typst Template
**Location**: `web_app/web/reports/pet_public_info.typ`

This template defines the visual layout of the pet profile card. It includes:
- Pet picture placeholder
- Pet name (prominent title)
- Lost status alert (if applicable)
- Basic info (breed, sex, weight, age)
- About section
- Contact info (if lost)
- Health information (spay/neuter status)

### 2. Image Rendering Function
**Location**: `web_app/src/api/pdf_handler.rs:create_image_bytes_from_str()`

Core function that:
- Takes Typst markup as string
- Compiles it using the Typst compiler
- Renders the first page to PNG
- Returns PNG bytes

Key parameters:
- `content`: The Typst markup (after Tera rendering)
- `pixel_per_pt`: Resolution control (default 2.0, higher = better quality)

### 3. Pet Public Info Generator
**Location**: `web_app/src/api/pet.rs:generate_public_info_image_bytes()`

High-level function that:
- Fetches pet data from database
- Prepares data for template
- Renders Tera template with pet data
- Calls image rendering function
- Returns PNG bytes

## Usage Example

```rust
use crate::api::pet::generate_public_info_image_bytes;

// In your endpoint or service
pub async fn handle_generate_pet_image(
    pet_id: i64,
    user_id: i64,
    repo: &repo::ImplAppRepo,
) -> Result<Vec<u8>> {
    // Generate the image
    let png_bytes = generate_public_info_image_bytes(
        pet_id,
        user_id,
        repo
    ).await?;

    // Return or save the image
    Ok(png_bytes)
}
```

## Creating an API Endpoint

To expose this via HTTP, add to your routes:

```rust
// In web_app/src/front/pet.rs or similar
#[get("/pet/image/{external_id}")]
pub async fn get_pet_public_image(
    external_id: web::Path<String>,
    repo: web::Data<repo::ImplAppRepo>,
    identity: Identity,
) -> Result<HttpResponse, PetInfoError> {
    let user_id = get_user_id_from_identity(&identity)?;
    let pet = repo.get_pet_by_external_id(&external_id).await?;

    let image_bytes = api::pet::generate_public_info_image_bytes(
        pet.id,
        user_id,
        &repo,
    ).await?;

    Ok(HttpResponse::Ok()
        .content_type("image/png")
        .body(image_bytes))
}
```

## Benefits of Typst over HTML-to-Image

1. **No Headless Browser**: Typst is native Rust, no need for Chrome/Chromium
2. **Fast**: Compiles and renders quickly
3. **Deterministic**: Same input always produces identical output
4. **High Quality**: Vector-based rendering, scales to any resolution
5. **Small Binary**: No external dependencies like browser engines
6. **Easy Styling**: Typst has intuitive layout and styling primitives

## Customization

### Changing Image Resolution

```rust
// Standard quality (file size ~100-200KB)
create_image_bytes_from_str(&content, Some(2.0))

// High quality for print (file size ~500KB-1MB)
create_image_bytes_from_str(&content, Some(3.0))

// Low quality for thumbnails (file size ~50KB)
create_image_bytes_from_str(&content, Some(1.0))
```

### Modifying the Template

Edit `web/reports/pet_public_info.typ` to change:
- Colors: `fill: rgb("#0f172a")`
- Fonts: `font: "PT Sans"`
- Layout: `#set page(width: 800pt, height: auto)`
- Spacing: `#v(20pt)` for vertical space

### Adding Pet Picture Support

Currently the template has a placeholder. To add real images:

1. Load the pet picture from storage
2. Save it temporarily or convert to base64
3. Use Typst's `#image()` function:

```typst
#image("path/to/pet.jpg", width: 100%)
```

## Dependencies

Added to `Cargo.toml`:
```toml
typst = "0.13.1"
typst-pdf = "0.13.1"
typst-render = "0.13.1"  # <-- New dependency for PNG rendering
typst-assets = { version = "0.13.1", features = ["fonts"] }
```

## Testing

```bash
# Run type checking
cargo check

# Run tests
cargo test

# Build the project
cargo build --release
```

## Future Enhancements

1. **Multiple Page Support**: Extend to render all pages, not just the first
2. **Image Embedding**: Support embedding actual pet photos in the card
3. **Multiple Formats**: Add JPEG, WebP output options
4. **Caching**: Cache generated images with pet data hash as key
5. **Batch Generation**: Generate images for multiple pets in parallel
6. **QR Code Integration**: Add QR code linking to pet profile (already have fast_qr crate!)
7. **Custom Themes**: Allow users to choose card color schemes

## Related Code

- `web_app/web/templates/pet_public_info.html` - Original HTML template
- `web_app/web/reports/pet_default.typ` - Existing PDF report template
- `web_app/src/api/pdf_handler.rs` - PDF and image generation functions
- `web_app/src/front/templates.rs` - Template loader configuration
