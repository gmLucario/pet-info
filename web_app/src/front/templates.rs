use std::sync::LazyLock;
use tera::Tera;

/// Global Tera template engine instance for web HTML templates.
///
/// This lazy-loaded static instance loads all HTML templates from the
/// `web/templates/` directory and its subdirectories. The templates are
/// compiled once at first access and cached for subsequent use.
pub static WEB_TEMPLATES: LazyLock<Tera> =
    LazyLock::new(|| Tera::new("web/templates/**/*.html").unwrap());

/// Global Tera template engine instance for PDF report templates.
///
/// This lazy-loaded static instance loads all Typst templates from the
/// `web/reports/` directory. These templates are used for generating
/// PDF reports using the Typst typesetting system.
pub static PDF_REPORT_TEMPLATES: LazyLock<Tera> =
    LazyLock::new(|| Tera::new("web/reports/**/*.typ").unwrap());

/// Global Tera template engine instance for blog markdown templates.
///
/// This lazy-loaded static instance loads all Markdown templates from the
/// `web/blog/` directory. These templates are used for rendering blog
/// content and posts.
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
