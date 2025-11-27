use std::sync::LazyLock;
use tera::Tera;

/// HTML templates from web/templates/
pub static WEB_TEMPLATES: LazyLock<Tera> =
    LazyLock::new(|| Tera::new("web/templates/**/*.html").unwrap());

/// Webmanifest templates
pub static WEB_MANIFESTS: LazyLock<Tera> =
    LazyLock::new(|| Tera::new("web/templates/**/*.webmanifest").unwrap());

/// Typst templates for PDF reports
pub static PDF_REPORT_TEMPLATES: LazyLock<Tera> =
    LazyLock::new(|| Tera::new("web/reports/**/*.typ").unwrap());

/// Markdown blog templates
pub static BLOG_TEMPLATES: LazyLock<Tera> = LazyLock::new(|| Tera::new("web/blog/*.md").unwrap());

#[cfg(test)]
mod tests {
    use rust_decimal::prelude::Zero;

    use super::*;

    #[test]
    fn test_web_templates_initialization() {
        // Test that WEB_TEMPLATES can be initialized without panicking
        let templates = &*WEB_TEMPLATES;
        assert!(!templates.get_template_names().count().is_zero());

        // Verify some expected templates exist
        assert!(templates.get_template("base.html").is_ok());
        assert!(templates.get_template("index.html").is_ok());
        assert!(templates.get_template("pet.html").is_ok());
    }

    #[test]
    fn test_pdf_report_templates_initialization() {
        // Test that PDF_REPORT_TEMPLATES can be initialized without panicking
        let templates = &*PDF_REPORT_TEMPLATES;
        assert!(
            !templates
                .get_template_names()
                .collect::<Vec<_>>()
                .is_empty()
        );

        // Verify expected report template exists
        assert!(templates.get_template("pet_default.typ").is_ok());
    }

    #[test]
    fn test_blog_templates_initialization() {
        // Test that BLOG_TEMPLATES can be initialized without panicking
        let templates = &*BLOG_TEMPLATES;
        assert!(
            !templates
                .get_template_names()
                .collect::<Vec<_>>()
                .is_empty()
        );

        // Verify some expected blog templates exist
        assert!(templates.get_template("about.md").is_ok());
        assert!(templates.get_template("privacy.md").is_ok());
        assert!(templates.get_template("terms.md").is_ok());
    }

    #[test]
    fn test_web_templates_load_nested_templates() {
        let templates = &*WEB_TEMPLATES;

        // Test that nested templates in subdirectories are loaded
        assert!(templates.get_template("errors/internal_error.html").is_ok());
        assert!(templates.get_template("errors/url_not_found.html").is_ok());
        assert!(templates.get_template("widgets/add_pet_form.html").is_ok());
        assert!(templates.get_template("widgets/pets.html").is_ok());
    }
}
