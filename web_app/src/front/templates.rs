use lazy_static::lazy_static;
use tera::Tera;

lazy_static! {
    pub static ref WEB_TEMPLATES: Tera = Tera::new("web/templates/**/*.html").unwrap();
}
