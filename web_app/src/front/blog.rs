use ntex::web;
use serde_json::json;
use std::sync::LazyLock;
use tera::Tera;

use crate::front::{errors, templates};
use pulldown_cmark::{Options, Parser};

pub static BLOG_TEMPLATES: LazyLock<Tera> = LazyLock::new(|| Tera::new("web/blog/*.md").unwrap());

#[derive(serde::Deserialize, serde::Serialize)]
struct Blog {
    title: String,
    content: String,
}

#[web::get("{blog_name}")]
async fn get_blog_entry(
    path: web::types::Path<(String,)>,
) -> Result<impl web::Responder, web::Error> {
    let blog_name = path.0.to_string();

    let markdown_input = BLOG_TEMPLATES
        .render(&format!("{}.md", blog_name), &tera::Context::new())
        .map_err(|_| errors::UserError::UrlNotFound)?;

    let mut html_output = String::new();
    pulldown_cmark::html::push_html(
        &mut html_output,
        Parser::new_ext(&markdown_input, Options::empty()),
    );

    let context = tera::Context::from_value(json!({
        "blog": Blog {
            title: blog_name,
            content: html_output,
        }
    }))
    .unwrap_or_default();

    let content = templates::WEB_TEMPLATES
        .render("blog.html", &context)
        .map_err(|e| {
            errors::ServerError::TemplateError(format!(
                "at /blog endpoint the template couldnt be rendered: {e}"
            ))
        })?;

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(content))
}
