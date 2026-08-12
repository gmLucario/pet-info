use chrono::{DateTime, NaiveDate, NaiveDateTime};
use chrono_tz::Tz;
use std::sync::LazyLock;
use tera::{Error, Kwargs, State, Tera};

fn date(value: &str, kwargs: Kwargs, _: &State) -> Result<String, Error> {
    let format = kwargs.get::<&str>("format")?.unwrap_or("%Y-%m-%d %H:%M:%S");

    if let Ok(date_time) = DateTime::parse_from_rfc3339(value) {
        if let Some(timezone) = kwargs.get::<&str>("timezone")? {
            let timezone = timezone
                .parse::<Tz>()
                .map_err(|_| Error::message(format!("Unknown timezone `{timezone}`")))?;
            return Ok(date_time
                .with_timezone(&timezone)
                .format(format)
                .to_string());
        }
        return Ok(date_time.format(format).to_string());
    }

    if let Ok(date_time) = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f") {
        return Ok(date_time.format(format).to_string());
    }

    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .map(|date| date.format(format).to_string())
        .map_err(|_| Error::message(format!("Unable to parse `{value}` as a date")))
}

fn filesizeformat(value: u64, _: Kwargs, _: &State) -> String {
    const UNITS: [&str; 5] = ["B", "kB", "MB", "GB", "TB"];
    let mut size = value as f64;
    let mut unit = 0;
    while size >= 1000.0 && unit < UNITS.len() - 1 {
        size /= 1000.0;
        unit += 1;
    }

    if unit == 0 {
        format!("{value} B")
    } else {
        format!("{size:.1} {}", UNITS[unit])
    }
}

/// Global Tera template engine instance for web HTML templates.
///
/// This lazy-loaded static instance loads all HTML templates from the
/// `web/templates/` directory and its subdirectories. The templates are
/// compiled once at first access and cached for subsequent use.
pub static WEB_TEMPLATES: LazyLock<Tera> = LazyLock::new(|| {
    let mut tera = Tera::default();
    tera.register_filter("date", date);
    tera.register_filter("filesizeformat", filesizeformat);
    tera.load_from_glob("web/templates/**/*.html").unwrap();
    tera
});

pub static WEB_MANIFESTS: LazyLock<Tera> = LazyLock::new(|| {
    let mut tera = Tera::default();
    tera.load_from_glob("web/templates/**/*.webmanifest")
        .unwrap();
    tera
});

/// Global Tera template engine instance for PDF report templates.
///
/// This lazy-loaded static instance loads all Typst templates from the
/// `web/reports/` directory. These templates are used for generating
/// PDF reports using the Typst typesetting system.
pub static PDF_REPORT_TEMPLATES: LazyLock<Tera> = LazyLock::new(|| {
    let mut tera = Tera::default();
    tera.register_filter("date", date);
    tera.load_from_glob("web/reports/**/*.typ").unwrap();
    tera
});

/// Global Tera template engine instance for blog markdown templates.
///
/// This lazy-loaded static instance loads all Markdown templates from the
/// `web/blog/` directory. These templates are used for rendering blog
/// content and posts.
pub static BLOG_TEMPLATES: LazyLock<Tera> = LazyLock::new(|| {
    let mut tera = Tera::default();
    tera.load_from_glob("web/blog/*.md").unwrap();
    tera
});

#[cfg(test)]
mod tests {
    use super::*;

    fn template_names(templates: &Tera) -> Vec<&str> {
        templates.get_template_names().collect()
    }

    #[test]
    fn test_web_templates_initialization() {
        // Test that WEB_TEMPLATES can be initialized without panicking
        let templates = &*WEB_TEMPLATES;
        let template_names = template_names(templates);
        assert!(template_names.contains(&"base.html"));
        assert!(template_names.contains(&"index.html"));
        assert!(template_names.contains(&"pet.html"));
    }

    #[test]
    fn test_pdf_report_templates_initialization() {
        // Test that PDF_REPORT_TEMPLATES can be initialized without panicking
        let templates = &*PDF_REPORT_TEMPLATES;
        let template_names = template_names(templates);
        assert!(!template_names.is_empty());

        // Verify expected report template exists
        assert!(template_names.contains(&"pet_default.typ"));
    }

    #[test]
    fn test_blog_templates_initialization() {
        // Test that BLOG_TEMPLATES can be initialized without panicking
        let templates = &*BLOG_TEMPLATES;
        let template_names = template_names(templates);
        assert!(!template_names.is_empty());

        // Verify some expected blog templates exist
        assert!(template_names.contains(&"about.md"));
        assert!(template_names.contains(&"privacy.md"));
        assert!(template_names.contains(&"terms.md"));
    }

    #[test]
    fn test_web_templates_load_nested_templates() {
        let templates = &*WEB_TEMPLATES;
        let template_names = template_names(templates);

        // Test that nested templates in subdirectories are loaded
        assert!(template_names.contains(&"errors/internal_error.html"));
        assert!(template_names.contains(&"errors/url_not_found.html"));
        assert!(template_names.contains(&"widgets/add_pet_form.html"));
        assert!(template_names.contains(&"widgets/pets.html"));
    }
}
