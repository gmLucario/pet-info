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

struct TypstWorld {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    font: Font,
    source: Source,
}

impl TypstWorld {
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
            Ok(self.source.clone())
        } else {
            Err(FileError::NotFound(id.vpath().as_rootless_path().into()))
        }
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
