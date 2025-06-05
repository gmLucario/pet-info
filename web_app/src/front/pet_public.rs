//! Handlers related to the /info/profile/{external-id} url

use ntex::web;
use serde_json::json;
use uuid::Uuid;

use crate::{
    api, consts,
    front::{AppState, errors, oauth, templates},
};

/// Renders a pet public info based on its `external_id`
/// If the pet_external_id is not linked to a pet; steps to link the
/// external id will be shown
#[web::get("/{pet_external_id}")]
async fn get_pet_info_view(
    app_state: web::types::State<AppState>,
    path: web::types::Path<(Uuid,)>,
    cookie: ntex_session::Session,
) -> Result<impl web::Responder, web::Error> {
    let pet_external_id = path.0;

    let external_id_metadata = api::pet::get_pet_external_id_metadata(
        &pet_external_id,
        &app_state.repo,
    )
    .await
    .map_err(|e| {
        errors::ServerError::InternalServerError(format!(
            "at /pet/external_id endpoint pet external id metadata couldnt be retrieved: {e}"
        ))
    })?;

    if external_id_metadata.is_none() {
        return Err(errors::UserError::UrlNotFound.into());
    }

    if external_id_metadata
        .map(|m| !m.is_linked)
        .unwrap_or_default()
    {
        return empty_tag_view(cookie, &pet_external_id);
    }

    let pet = api::pet::get_pet_public_info(pet_external_id, &app_state.repo)
        .await
        .map_err(|e| {
            errors::ServerError::InternalServerError(format!(
                "at /pet/external_id endpoint pet info couldnt be retrieved: {e}"
            ))
        })?;

    let context = tera::Context::from_value(json!({
        "pet": pet,
        "owner_contacts": api::user::get_owner_contacts(0, Some(pet_external_id), &app_state.repo)
        .await
        .map_err(|e| {
            errors::ServerError::InternalServerError(format!(
                "function get_owner_contacts raised an error: {e}"
            ))
        })?
    }))
    .unwrap_or_default();

    let content = templates::WEB_TEMPLATES
        .render("pet_public_info.html", &context)
        .map_err(|e| {
            errors::ServerError::TemplateError(format!(
                "at /pet/external_id endpoint the template couldnt be rendered: {e}"
            ))
        })?;

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(content))
}

fn empty_tag_view(
    cookie: ntex_session::Session,
    pet_external_id: &Uuid,
) -> Result<web::HttpResponse, web::Error> {
    let (auth_url, csrf_state) = oauth::get_new_auth_url();

    cookie
        .set(consts::CSRF_STATE_COOKIE_NAME, csrf_state)
        .map_err(|e| {
            errors::ServerError::InternalServerError(format!(
                "at pet_empty_tag.html cant set CSRF_STATE_COOKIE_NAME: {e}"
            ))
        })?;

    cookie
        .set(
            consts::REDIRECT_TO_COOKIE_NAME,
            format!("/pet/new?pet_external_id={pet_external_id}"),
        )
        .map_err(|e| {
            errors::ServerError::InternalServerError(format!(
                "at pet_empty_tag.html cant set REDIRECT_TO_COOKIE_NAME: {e}"
            ))
        })?;

    let context = tera::Context::from_value(json!({
        "google_outh_auth_url": &auth_url,
        "pet_external_id": pet_external_id,
    }))
    .unwrap_or_default();

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(
            templates::WEB_TEMPLATES
                .render("pet_empty_tag.html", &context)
                .map_err(|e| {
                    errors::ServerError::TemplateError(format!(
                        "at /pet/external_id endpoint the template couldnt be rendered: {e}"
                    ))
                })?,
        ))
}
