use crate::{
    api,
    front::{
        AppState, errors, forms,
        middleware::{self, logged_user::IsUserLoggedAndCanEdit},
        session, templates,
    },
    models,
};
use ntex::web;
use serde_json::json;

#[derive(serde::Deserialize)]
struct HealthPath {
    pet_external_id: uuid::Uuid,
    record_type: models::pet::PetHealthType,
}

#[web::get("{pet_external_id}/{record_type}")]
async fn get_pet_health_view(
    IsUserLoggedAndCanEdit(can_edit, user_id): IsUserLoggedAndCanEdit,
    app_state: web::types::State<AppState>,
    path: web::types::Path<HealthPath>,
) -> Result<impl web::Responder, web::Error> {
    let context = tera::Context::from_value(json!({
        "can_edit": &can_edit,
        "record_type": &path.record_type,
        "pet_external_id": &path.pet_external_id,
        "health_records": api::pet::get_pet_health_records(
            path.pet_external_id,
            &path.record_type,
            user_id,
            &app_state.repo,
        )
        .await
        .map_err(|e| {
            errors::ServerError::InternalServerError(format!(
                "function get_pet_health_records raised an error: {e}"
            ))
        })?,
    }))
    .unwrap_or_default();

    let content = templates::WEB_TEMPLATES
        .render("health_record.html", &context)
        .map_err(|e| {
            errors::ServerError::TemplateError(format!(
                "at /pet/health/{{pet_external_id}}/{{record_type}} endpoint the template couldnt be rendered: {e}"
            ))
        })?;

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(content))
}

/// Render pet health records table
#[web::get("{pet_external_id}/{record_type}/tbody")]
async fn pet_health_records(
    IsUserLoggedAndCanEdit(can_edit, user_id): IsUserLoggedAndCanEdit,
    app_state: web::types::State<AppState>,
    path: web::types::Path<HealthPath>,
) -> Result<impl web::Responder, web::Error> {
    let context = tera::Context::from_value(json!({
        "can_edit": &can_edit,
        "record_type": &path.record_type,
        "pet_external_id": &path.pet_external_id,
        "health_records": api::pet::get_pet_health_records(
            path.pet_external_id,
            &path.record_type,
            user_id,
            &app_state.repo,
        )
        .await
        .map_err(|e| {
            errors::ServerError::InternalServerError(format!(
                "function get_pet_health_records raised an error: {e}"
            ))
        })?,
    }))
    .unwrap_or_default();

    let content = templates::WEB_TEMPLATES
        .render("widgets/tbody_health_record.html", &context)
        .map_err(|e| {
            errors::ServerError::TemplateError(format!(
                "at /pet/health/{{pet_external_id}}/{{record_type}}/tbody endpoint the template couldnt be rendered: {e}"
            ))
        })?;

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(content))
}

/// Add health record to pet
#[web::post("{pet_external_id}/{record_type}/add")]
async fn add_health_record(
    _: middleware::logged_user::CheckUserCanAccessService,
    session::WebAppSession { user, .. }: session::WebAppSession,
    path: web::types::Path<HealthPath>,
    form: web::types::Form<forms::pet::HealthRecordForm>,
    app_state: web::types::State<AppState>,
    _: middleware::csrf_token::CsrfToken,
) -> Result<impl web::Responder, web::Error> {
    let form = forms::pet::HealthRecordForm {
        value: ammonia::clean(&form.value),
        date: form.date,
    };

    let desc = form.value.to_string();
    if path.record_type.eq(&models::pet::PetHealthType::Weight) {
        desc.parse::<f64>()
            .map_err(|_| errors::UserError::FormInputValueError("peso no es numerico".into()))?;
    }

    api::pet::insert_pet_health_record(
        path.pet_external_id,
        &path.record_type,
        user.id,
        desc,
        form.date,
        &app_state.repo,
    )
    .await
    .map_err(|e| {
        errors::ServerError::InternalServerError(format!(
            "function add_health_record raised an error: {e}"
        ))
    })?;

    Ok(web::HttpResponse::Created()
        .set_header("HX-Trigger", "healthRecordUpdated")
        .content_type("text/html; charset=utf-8")
        .finish())
}

#[derive(serde::Deserialize)]
struct HealthDeletePath {
    record_id: i64,
    pet_external_id: uuid::Uuid,
    record_type: models::pet::PetHealthType,
}

/// Delete health record from pet
#[web::delete("{pet_external_id}/{record_type}/{record_id}")]
async fn delete_health_record(
    _: middleware::logged_user::CheckUserCanAccessService,
    path: web::types::Path<HealthDeletePath>,
    session::WebAppSession { user, .. }: session::WebAppSession,
    app_state: web::types::State<AppState>,
    _: middleware::csrf_token::CsrfToken,
) -> Result<impl web::Responder, web::Error> {
    api::pet::delete_pet_health_record(
        path.record_id,
        path.pet_external_id,
        user.id,
        &path.record_type,
        &app_state.repo,
    )
    .await
    .map_err(|e| {
        errors::ServerError::InternalServerError(format!(
            "function delete_pet_health_record raised an error: {e}"
        ))
    })?;

    Ok(web::HttpResponse::Created()
        .set_header("HX-Trigger", "healthRecordUpdated")
        .content_type("text/html; charset=utf-8")
        .finish())
}
