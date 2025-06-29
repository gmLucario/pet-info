//! PDF generation using Typst typesetting system.

use anyhow::Result;
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
struct TypstWorld {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    font: Font,
    source: Source,
}

impl TypstWorld {
    /// Creates a new TypstWorld instance with the given text.
    fn new(text: &str) -> Result<Self> {
        let font_data = typst_assets::fonts()
            .next()
            .ok_or_else(|| anyhow::anyhow!("No fonts available"))?;
        let font = Font::new(Bytes::new(font_data), 0)
            .ok_or_else(|| anyhow::anyhow!("Font creation failed"))?;

        Ok(Self {
            library: LazyHash::new(Library::default()),
            book: LazyHash::new(FontBook::from_fonts([&font])),
            font,
            source: Source::detached(text),
        })
    }
}

impl World for TypstWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        self.source.id()
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.source.id() {
            return Ok(self.source.clone());
        }

        Err(FileError::NotFound(id.vpath().as_rootless_path().into()))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        Err(FileError::NotFound(id.vpath().as_rootless_path().into()))
    }

    fn font(&self, _: usize) -> Option<Font> {
        Some(self.font.clone())
    }

    fn today(&self, _: Option<i64>) -> Option<Datetime> {
        None
    }
}

/// Converts Typst markup content to PDF bytes.
///
/// # Arguments
/// * `content` - Typst markup text to compile
///
/// # Returns
/// PDF document as bytes
///
/// # Errors
/// Returns error if compilation or PDF generation fails
pub fn create_pdf_bytes_from_str(content: &str) -> Result<Vec<u8>> {
    let world = TypstWorld::new(content)?;
    let document = typst::compile(&world)
        .output
        .map_err(|e| anyhow::anyhow!("Compilation failed: {:?}", e))?;
    typst_pdf::pdf(&document, &PdfOptions::default())
        .map_err(|e| anyhow::anyhow!("PDF generation failed: {:?}", e))
}
