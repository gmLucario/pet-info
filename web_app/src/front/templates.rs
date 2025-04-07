use std::sync::LazyLock;
use tera::Tera;

pub static WEB_TEMPLATES: LazyLock<Tera> =
    LazyLock::new(|| Tera::new("web/templates/**/*.html").unwrap());

pub static PDF_REPORT_TEMPLATES: LazyLock<Tera> =
    LazyLock::new(|| Tera::new("web/reports/**/*.typ").unwrap());
