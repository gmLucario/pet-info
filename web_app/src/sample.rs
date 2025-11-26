use qrcode::{QrCode, Version, EcLevel};
use svg::node::element::{Circle, Rectangle, Group};
use svg::Document;

fn main() {
    // 1. CONFIGURATION
    let data = "https://pet-info.link/info/411d4fa6-4347-4ed0-983c-f253d82f3b7e";
    let output_file = "liquid_qr.svg";

    // Generate the QR code matrix
    // We use Quartile (Q) or High (H) error correction to allow for a logo later
    let code = QrCode::with_error_correction_level(data, EcLevel::L).unwrap();
    let width = code.width();

    // 2. SETUP SVG
    // Define a "quiet zone" (padding) around the code
    let padding = 4;
    let canvas_size = width + (padding * 2);

    let mut document = Document::new()
        .set("viewBox", (0, 0, canvas_size, canvas_size))
        .set("style", "background-color: white;");

    // 3. DEFINE THE LIQUID GROUP
    // This group will hold all the connected blobs
    let mut liquid_group = Group::new().set("fill", "#0f172a"); // Dark Blue/Black

    // 4. HELPER: CHECK IF MODULE IS PART OF A FINDER PATTERN (THE 3 EYES)
    // Finder patterns are 7x7 squares at (0,0), (width-7, 0), and (0, width-7)
    let is_finder = |x: usize, y: usize| -> bool {
        (x < 7 && y < 7) ||                       // Top-Left
        (x >= width - 7 && y < 7) ||              // Top-Right
        (x < 7 && y >= width - 7)                 // Bottom-Left
    };

    // 5. DRAW THE LIQUID BODY
    for y in 0..width {
        for x in 0..width {
            // qrcode crate allows easy indexing with [(x,y)]
            // It returns a Color enum (Light/Dark). We check if it is Dark.
            if let qrcode::Color::Dark = code[(x, y)] {
                
                // Skip if this is part of the finder patterns (we draw them manually later)
                if is_finder(x, y) { continue; }

                // Adjust coordinates for padding
                let cx = x + padding;
                let cy = y + padding;

                // A. Draw the MAIN CIRCLE for this module
                let circle = Circle::new()
                    .set("cx", cx as f32 + 0.5)
                    .set("cy", cy as f32 + 0.5)
                    .set("r", 0.5); // Radius 0.5 = Diameter 1.0 (fills the cell)
                liquid_group = liquid_group.add(circle);

                // B. DRAW BRIDGES (The "Liquid" Logic)
                // If the right neighbor is also Dark, draw a rectangle connecting them
                if x + 1 < width {
                    if let qrcode::Color::Dark = code[(x + 1, y)] {
                        if !is_finder(x + 1, y) {
                            let h_bridge = Rectangle::new()
                                .set("x", cx as f32 + 0.5)
                                .set("y", cy as f32)
                                .set("width", 1) // Bridge to the center of next cell
                                .set("height", 1);
                            liquid_group = liquid_group.add(h_bridge);
                        }
                    }
                }

                // If the bottom neighbor is also Dark, draw a rectangle connecting them
                if y + 1 < width {
                    if let qrcode::Color::Dark = code[(x, y + 1)] {
                        if !is_finder(x, y + 1) {
                            let v_bridge = Rectangle::new()
                                .set("x", cx as f32)
                                .set("y", cy as f32 + 0.5)
                                .set("width", 1)
                                .set("height", 1);
                            liquid_group = liquid_group.add(v_bridge);
                        }
                    }
                }
            }
        }
    }

    document = document.add(liquid_group);

    // 6. DRAW CUSTOM "SQUIRCLE" FINDER PATTERNS
    // Function to draw one eye at a specific (tx, ty)
    let draw_eye = |tx: usize, ty: usize| -> Group {
        let mut group = Group::new().set("fill", "#0f172a");
        let x = tx + padding;
        let y = ty + padding;

        // Outer Box (7x7) with heavy rounding
        let outer = Rectangle::new()
            .set("x", x)
            .set("y", y)
            .set("width", 7)
            .set("height", 7)
            .set("rx", 2.5) // High corner radius makes it a "Squircle"
            .set("ry", 2.5);

        // White Mask (The gap)
        let gap = Rectangle::new()
            .set("x", x as f32 + 1.0)
            .set("y", y as f32 + 1.0)
            .set("width", 5)
            .set("height", 5)
            .set("rx", 2.0)
            .set("ry", 2.0)
            .set("fill", "white");

        // Inner Dot (3x3)
        let inner = Rectangle::new()
            .set("x", x as f32 + 2.0)
            .set("y", y as f32 + 2.0)
            .set("width", 3)
            .set("height", 3)
            .set("rx", 1.2)
            .set("ry", 1.2);

        group.add(outer).add(gap).add(inner)
    };

    // Add the three eyes
    document = document.add(draw_eye(0, 0));                 // Top-Left
    document = document.add(draw_eye(width - 7, 0));         // Top-Right
    document = document.add(draw_eye(0, width - 7));         // Bottom-Left

    // 7. SAVE
    svg::save(output_file, &document).unwrap();
    println!("Liquid QR saved to {}", output_file);
}