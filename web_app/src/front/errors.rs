use super::templates;
use derive_more::{Display, Error};
use log::error;
use ntex::{http, web};

#[derive(Debug, Display, Error)]
pub enum UserError {
    UrlNotFound,
    Unauthorized,
    NeedSubscription,
    FormInputValueError(#[error(not(source))] String),
}

impl web::error::WebResponseError for UserError {
    fn error_response(&self, _: &web::HttpRequest) -> web::HttpResponse {
        let mut context = tera::Context::new();
        error!("{:#?}", self);

        let template_name = match self {
            UserError::UrlNotFound => {
                context.insert("msg_details", "recurso no encontrado");
                "errors/url_not_found.html"
            }
            UserError::Unauthorized => {
                context.insert("msg_details", "favor de iniciar sesion");
                "errors/need_login.html"
            }
            UserError::NeedSubscription => {
                context.insert("msg_details", "su perido de prueba a terminado");
                "errors/need_subscription.html"
            }
            UserError::FormInputValueError(msg) => {
                context.insert(
                    "msg_details",
                    &format!("formulario con valores invalidos: {}", msg),
                );
                context.insert("form_url", "/pet/new");
                "errors/invalid_input_values.html"
            }
        };

        web::HttpResponse::build(self.status_code())
            .set_header("content-type", "text/html; charset=utf-8")
            .body(
                templates::WEB_TEMPLATES
                    .render(template_name, &context)
                    .unwrap_or(self.to_string()),
            )
    }

    fn status_code(&self) -> http::StatusCode {
        match *self {
            UserError::UrlNotFound => http::StatusCode::NOT_FOUND,
            UserError::Unauthorized => http::StatusCode::UNAUTHORIZED,
            UserError::NeedSubscription => http::StatusCode::PAYMENT_REQUIRED,
            UserError::FormInputValueError(_) => http::StatusCode::BAD_REQUEST,
        }
    }
}

#[derive(Debug, Display, Error)]
pub enum ServerError {
    TemplateError(#[error(not(source))] String),
    WidgetTemplateError(#[error(not(source))] String),
    ExternalServiceError(#[error(not(source))] String),
    InternalServerError(#[error(not(source))] String),
    InvalidCsrfToken,
}

impl ServerError {
    fn get_error_message(&self) -> String {
        match self {
            ServerError::TemplateError(msg) => format!("[TemplateError] {:#?}", msg),
            ServerError::WidgetTemplateError(msg) => format!("[WidgetTemplateError] {:#?}", msg),
            ServerError::ExternalServiceError(msg) => format!("[ExternalServiceError] {:#?}", msg),
            ServerError::InternalServerError(msg) => format!("[InternalServerError] {:#?}", msg),
            ServerError::InvalidCsrfToken => "[InvalidCsrfToken]".to_string(),
        }
    }
}

impl web::error::WebResponseError for ServerError {
    fn error_response(&self, _: &web::HttpRequest) -> web::HttpResponse {
        error!("{}", self.get_error_message());

        let template_name = match self {
            // will be a success status code cause it htmx should render something
            ServerError::WidgetTemplateError(_) => "errors/widget_page_err.html",
            _ => "errors/internal_error.html",
        };

        web::HttpResponse::build(self.status_code())
            .set_header("content-type", "text/html; charset=utf-8")
            .body(
                templates::WEB_TEMPLATES
                    .render(template_name, &tera::Context::new())
                    .unwrap_or(self.to_string()),
            )
    }

    fn status_code(&self) -> http::StatusCode {
        match *self {
            // will be a success status code cause it htmx should render something
            ServerError::WidgetTemplateError(_) => http::StatusCode::ACCEPTED,
            _ => http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
