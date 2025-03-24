use ntex::web;
use serde_json::json;

use crate::{
    api,
    front::{errors, middleware, templates, AppState},
    models,
};

use super::forms;

#[web::get("{pet_id}")]
async fn get_pet_notes_view(
    middleware::logged_user::IsUserLoggedAndCanEdit(can_edit, user_id): middleware::logged_user::IsUserLoggedAndCanEdit,
    params: web::types::Path<(i64,)>,
    app_state: web::types::State<AppState>,
) -> Result<impl web::Responder, web::Error> {
    let pet_id = params.0;
    let context = tera::Context::from_value(json!({
        "can_edit": can_edit,
        "pet_id": pet_id,
        "notes": if let Some(user_id) = user_id {
            api::pet::get_pet_notes(user_id, pet_id, &app_state.repo).await
            .map_err(|e| {
                errors::ServerError::InternalServerError(format!(
                    "function get_pet_notes raised an error: {}",
                    e
                ))
            })?
        } else {
            vec![]
        },
    }))
    .unwrap_or_default();

    let content = templates::WEB_TEMPLATES
        .render("pet_note.html", &context)
        .map_err(|e| {
            errors::ServerError::TemplateError(format!(
                "at /pet endpoint the template couldnt be rendered: {}",
                e
            ))
        })?;

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(content))
}

#[web::post("{pet_id}")]
async fn new_pet_note(
    _: middleware::logged_user::CheckUserCanAccessService,
    logged_user: models::user_app::User,
    params: web::types::Path<(i64,)>,
    form: web::types::Form<forms::pet::PetNoteForm>,
    app_state: web::types::State<AppState>,
    _: middleware::csrf_token::CsrfToken,
) -> Result<impl web::Responder, web::Error> {
    let pet_id = params.0;
    let form = forms::pet::PetNoteForm {
        title: ammonia::clean(&form.0.title),
        body: ammonia::clean(&form.0.body),
    };

    api::pet::add_new_note(logged_user.id, pet_id, form.into(), &app_state.repo)
        .await
        .map_err(|e| {
            errors::ServerError::InternalServerError(format!(
                "function add_new_note raised an error: {}",
                e
            ))
        })?;

    Ok(web::HttpResponse::Created()
        .set_header("HX-Trigger", "petNoteRecordUpdated")
        .body(""))
}

#[web::get("{pet_id}/list")]
async fn get_pet_notes(
    middleware::logged_user::IsUserLoggedAndCanEdit(can_edit, user_id): middleware::logged_user::IsUserLoggedAndCanEdit,
    params: web::types::Path<(i64,)>,
    app_state: web::types::State<AppState>,
) -> Result<impl web::Responder, web::Error> {
    if !can_edit {
        return Ok(web::HttpResponse::PaymentRequired()
            .content_type("text/html; charset=utf-8")
            .body(""));
    }

    let pet_id = params.0;
    let context = tera::Context::from_value(json!({
        "pet_id": pet_id,
        "notes": if let Some(user_id) = user_id {
            api::pet::get_pet_notes(user_id, pet_id, &app_state.repo).await
            .map_err(|e| {
                errors::ServerError::InternalServerError(format!(
                    "function get_pet_notes raised an error: {}",
                    e
                ))
            })?
        } else {
            vec![]
        },
    }))
    .unwrap_or_default();

    let content = templates::WEB_TEMPLATES
        .render("widgets/pet_notes.html", &context)
        .map_err(|e| {
            errors::ServerError::TemplateError(format!(
                "at /pet/note/<pet_id>/list endpoint the template couldnt be rendered: {}",
                e
            ))
        })?;

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(content))
}

#[web::delete("{pet_id}/delete/{note_id}")]
async fn delete_pet_note(
    _: middleware::logged_user::CheckUserCanAccessService,
    logged_user: models::user_app::User,
    params: web::types::Path<(i64, i64)>,
    app_state: web::types::State<AppState>,
) -> Result<impl web::Responder, web::Error> {
    api::pet::delete_note(params.1, params.0, logged_user.id, &app_state.repo)
        .await
        .map_err(|e| {
            errors::ServerError::InternalServerError(format!(
                "function delete_note raised an error: {}",
                e
            ))
        })?;

    Ok(web::HttpResponse::Ok()
        .set_header("HX-Trigger", "petNoteRecordUpdated")
        .body(""))
}
