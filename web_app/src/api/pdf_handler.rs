//! # PDF Handler API Module
//!
//! This module provides PDF generation capabilities using the Typst typesetting system.
//! It enables creating PDF documents from markup text for reports, certificates,
//! and other pet-related documentation.

use anyhow::{Result, bail};
use typst::{
    Library, World,
    diag::{FileError, FileResult},
    foundations::{Bytes, Datetime},
    syntax::{FileId, Source},
    text::{Font, FontBook},
    utils::LazyHash,
};
use typst_pdf::PdfOptions;

/// Typst world implementation for PDF compilation.
///
/// This struct provides the necessary context and resources for Typst
/// to compile markup text into PDF documents. It manages fonts, library
/// access, and source files required for the compilation process.
struct TypstWorld {
    /// Typst standard library functions and definitions
    library: LazyHash<Library>,
    /// Font book containing available fonts for rendering
    book: LazyHash<FontBook>,
    /// Primary font used for text rendering
    font: Font,
    /// Source content to be compiled
    source: Source,
}

impl TypstWorld {
    /// Creates a new TypstWorld instance for PDF compilation.
    ///
    /// Initializes the Typst compilation environment with default fonts
    /// and library settings. This prepares the world for processing
    /// Typst markup into PDF format.
    ///
    /// # Arguments
    /// * `text` - The Typst markup content to be compiled
    ///
    /// # Returns
    /// * `Result<Self>` - Configured TypstWorld instance or error
    ///
    /// # Errors
    /// Returns an error if typst_assets fonts cannot be loaded or
    /// if font initialization fails.
    fn new(text: &str) -> Result<Self> {
        let typst_world = typst_assets::fonts().next().map(|font| {
            Font::new(Bytes::new(font), 0).map(|f| Self {
                library: LazyHash::new(Library::default()),
                book: LazyHash::new(FontBook::from_fonts([&f])),
                font: f,
                source: Source::detached(text),
            })
        });

        if let Some(Some(world)) = typst_world {
            return Ok(world);
        }

        bail!("typst_assets couldnt be loaded")
    }
}

/// Implementation of Typst's World trait for PDF compilation.
///
/// This implementation provides Typst with access to necessary resources
/// like fonts, library functions, and source files during compilation.
impl World for TypstWorld {
    /// Returns reference to the Typst standard library.
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    /// Returns reference to the font book containing available fonts.
    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    /// Returns the main source file ID for compilation.
    fn main(&self) -> FileId {
        self.source.id()
    }

    /// Provides source content for a given file ID.
    ///
    /// # Arguments
    /// * `id` - File identifier to retrieve source for
    ///
    /// # Returns
    /// * `FileResult<Source>` - Source content or file not found error
    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.source.id() {
            Ok(self.source.clone())
        } else {
            Err(FileError::NotFound(id.vpath().as_rootless_path().into()))
        }
    }

    /// Provides file content for a given file ID.
    ///
    /// Currently returns NotFound for all files as we only work with
    /// the main source content.
    ///
    /// # Arguments
    /// * `id` - File identifier to retrieve content for
    ///
    /// # Returns
    /// * `FileResult<Bytes>` - Always returns NotFound error
    fn file(&self, id: FileId) -> FileResult<Bytes> {
        Err(FileError::NotFound(id.vpath().as_rootless_path().into()))
    }

    /// Returns a font by index.
    ///
    /// # Arguments
    /// * `_` - Font index (unused, always returns the primary font)
    ///
    /// # Returns
    /// * `Option<Font>` - Clone of the primary font
    fn font(&self, _: usize) -> Option<Font> {
        Some(self.font.clone())
    }

    /// Returns current date/time for document compilation.
    ///
    /// # Arguments
    /// * `_` - Optional timezone offset (unused)
    ///
    /// # Returns
    /// * `Option<Datetime>` - Always returns None (no date functions)
    fn today(&self, _: Option<i64>) -> Option<Datetime> {
        None
    }
}

/// Creates PDF bytes from Typst markup content.
///
/// This is the main public function for PDF generation. It takes Typst markup
/// text and compiles it into a PDF document, returning the PDF as bytes.
///
/// # Arguments
/// * `content` - Typst markup content to compile into PDF
///
/// # Returns
/// * `Result<Vec<u8>>` - PDF document as bytes or compilation error
///
/// # Process
/// 1. Create TypstWorld with the provided content
/// 2. Compile the content using Typst compiler
/// 3. Convert compiled document to PDF format
/// 4. Return PDF bytes
///
/// # Errors
/// Returns an error if:
/// - TypstWorld creation fails (font loading issues)
/// - Typst compilation fails (syntax errors, etc.)
/// - PDF generation fails
///
/// # Example
/// ```rust
/// let typst_content = r#"
/// = Pet Health Report
/// *Pet Name:* Fluffy
/// *Age:* 3 years
/// "#;
/// let pdf_bytes = create_pdf_bytes_from_str(typst_content)?;
/// std::fs::write("report.pdf", pdf_bytes)?;
/// ```
pub fn create_pdf_bytes_from_str(content: &str) -> Result<Vec<u8>> {
    let internal_typst_world = TypstWorld::new(content)?;
    let doc = typst::compile(&internal_typst_world)
        .output
        .map(|document| typst_pdf::pdf(&document, &PdfOptions::default()));

    if let Ok(Ok(buffer)) = doc {
        return Ok(buffer);
    }

    bail!("typst content couldnt be compiled")
}
