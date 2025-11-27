//! Pet management HTTP handlers

use anyhow::{Context, bail};
use futures::{TryStreamExt, future::ok, stream::once};
use ntex::{util::Bytes, web};
use serde_json::json;
use std::str::FromStr;
use uuid::Uuid;

use crate::{
    api, config, consts,
    front::{AppState, errors, forms, middleware, session, templates, utils},
};

/// Extract header value as string, returns empty if not found
fn get_header_str_value(headers: &ntex::http::HeaderMap, key: &str) -> String {
    let default_header_value = ntex::http::header::HeaderValue::from_static("");

    headers
        .get(key)
        .unwrap_or(&default_header_value)
        .to_str()
        .unwrap_or_default()
        .to_string()
}

/// Check if multipart field is a pet picture upload
fn is_image_field(field: &ntex_multipart::Field, content_disposition: &str) -> bool {
    field.content_type().essence_str().contains("image") && content_disposition.contains("pet_pic")
}

/// Process and validate image field
async fn process_image_field(field: ntex_multipart::Field) -> anyhow::Result<forms::pet::Pic> {
    let body = utils::get_bytes_value(field).await;

    // Validate image size
    if body.len() > consts::PIC_PET_MAX_SIZE_BYTES {
        bail!(
            "Image is too big. Maximum size: {} bytes",
            consts::PIC_PET_MAX_SIZE_BYTES
        );
    }

    let extension = crate::utils::detect_image_format(&body);

    Ok(forms::pet::Pic {
        filename_extension: extension.to_string(),
        body,
    })
}

/// Parse multipart pet form with image upload and cropping
async fn deserialize_pet_form(
    mut payload: ntex_multipart::Multipart,
) -> anyhow::Result<super::forms::pet::CreatePetForm> {
    let _span = logfire::span!("deserialize_pet_form").entered();

    let mut form = forms::pet::CreatePetForm::default();
    let mut cropper_box: Option<forms::pet::CropperBox> = None;
    let mut pet_pic: Option<forms::pet::Pic> = None;

    while let Ok(Some(field)) = payload.try_next().await {
        let headers = field.headers();

        let content_disposition = get_header_str_value(headers, "content-disposition");

        if is_image_field(&field, &content_disposition) {
            pet_pic = Some(process_image_field(field).await?);
            continue;
        }

        let field_value = ammonia::clean(&utils::get_field_value(field).await);

        if content_disposition.contains("pet_full_name") {
            form.pet_full_name = field_value;
        } else if content_disposition.contains("pet_birthday") {
            form.pet_birthday = chrono::NaiveDate::parse_from_str(&field_value, "%Y-%m-%d")?;
        } else if content_disposition.contains("pet_breed") {
            form.pet_breed = field_value;
        } else if content_disposition.contains("is_lost") {
            form.is_lost = field_value.contains("on");
        } else if content_disposition.contains("is_spaying_neutering") {
            form.is_spaying_neutering = field_value.contains("on");
        } else if content_disposition.contains("is_female") {
            form.is_female = field_value.contains("on");
        } else if content_disposition.contains("about_pet") {
            form.about_pet = field_value;
        } else if content_disposition.contains("pet_external_id") {
            form.pet_external_id = Some(Uuid::from_str(&field_value).unwrap_or(Uuid::new_v4()));
        } else if content_disposition.contains("cropper_box") {
            cropper_box = serde_json::from_str(&field_value)?;
        }
    }

    if let (Some(cropper_box), Some(pet_pic)) = (cropper_box, pet_pic) {
        form.pet_pic = Some(forms::pet::PetPic {
            filename_extension: "png".to_string(),
            body: utils::crop_circle(&pet_pic, cropper_box.x, cropper_box.y, cropper_box.diameter)?,
        })
    }

    Ok(form)
}

/// Render pet dashboard with user's pets
#[web::get("")]
async fn get_pet_view(
    session::WebAppSession { user, .. }: session::WebAppSession,
    app_state: web::types::State<AppState>,
) -> Result<impl web::Responder, web::Error> {
    let context = tera::Context::from_value(json!({
        "pets": api::pet::get_user_pets_cards(user.id, &app_state.repo)
        .await
        .map_err(|e| {
            errors::ServerError::InternalServerError(format!(
                "function get_user_pets_cards raised an error: {e}"
            ))
        })?,
    }))
    .unwrap_or_default();

    let content = templates::WEB_TEMPLATES
        .render("pet.html", &context)
        .map_err(|e| {
            errors::ServerError::TemplateError(format!(
                "at /pet endpoint the template couldnt be rendered: {e}"
            ))
        })?;

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(content))
}

/// HTMX endpoint for pets list widget
#[web::get("/list")]
async fn user_pets_list(
    session::WebAppSession { user, .. }: session::WebAppSession,
    app_state: web::types::State<AppState>,
) -> Result<impl web::Responder, web::Error> {
    let context = tera::Context::from_value(json!({
        "pets": api::pet::get_user_pets_cards(user.id, &app_state.repo)
        .await
        .map_err(|e| {
            errors::ServerError::InternalServerError(format!(
                "function get_user_pets_cards raised an error: {e}"
            ))
        })?,
    }))
    .unwrap_or_default();

    let content = templates::WEB_TEMPLATES
        .render("widgets/pets.html", &context)
        .map_err(|e| {
            errors::ServerError::TemplateError(format!(
                "at /pet/list endpoint the template couldnt be rendered: {e}"
            ))
        })?;

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(content))
}

#[derive(serde::Deserialize, Debug)]
struct PetFormQueryParams {
    pet_external_id: Option<Uuid>,
}

/// Render pet creation form
#[web::get("/new")]
async fn render_pet_details_form(
    _: session::WebAppSession,
    q: web::types::Query<PetFormQueryParams>,
) -> Result<impl web::Responder, web::Error> {
    let content = templates::WEB_TEMPLATES
        .render(
            "pet_details.html",
            &tera::Context::from_value(json!({
                "PIC_PET_MAX_SIZE_BYTES": consts::PIC_PET_MAX_SIZE_BYTES,
                "ACCEPTED_IMAGE_EXTENSIONS": consts::ACCEPTED_IMAGE_EXTENSIONS,
                "pet_external_id": q.pet_external_id,
            }))
            .unwrap_or_default(),
        )
        .map_err(|e| {
            errors::ServerError::TemplateError(format!(
                "at /pet/new endpoint the template couldnt be rendered: {e}"
            ))
        })?;

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(content))
}

/// Handle pet creation form submission
#[web::post("/new")]
async fn create_pet_request(
    _: middleware::csrf_token::CsrfToken,
    mut user_session: session::WebAppSession,
    app_state: web::types::State<AppState>,
    payload: ntex_multipart::Multipart,
    identity: ntex_identity::Identity,
) -> Result<impl web::Responder, web::Error> {
    let _span = logfire::span!("create_pet_request").entered();

    let pet_form = deserialize_pet_form(payload)
        .await
        .map_err(|e| errors::UserError::FormInputValueError(e.to_string()))?;

    let request_has_pet_external_id = pet_form.pet_external_id.is_some();

    if !(request_has_pet_external_id
        || user_session.has_pet_balance() && user_session.user.can_access_service())
    {
        return Err(errors::UserError::NeedSubscription.into());
    }

    api::pet::add_new_pet_to_user(
        api::pet::UserStateAddNewPet {
            user_id: user_session.user.id,
            user_email: user_session.user.email.to_string(),
            pet_balance: user_session.add_pet_balance,
        },
        pet_form,
        &app_state.repo,
        &app_state.storage_service,
    )
    .await
    .map_err(|e| errors::ServerError::InternalServerError(e.to_string()))?;

    user_session.add_pet_balance -=
        u32::from(!request_has_pet_external_id && user_session.add_pet_balance > 0);
    user_session.user.is_subscribed = true;

    identity.remember(serde_json::to_string(&user_session)?);

    utils::redirect_to("/pet")
}

/// Delete pet and all associated data
#[web::delete("/delete/{pet_id}")]
async fn delete_pet(
    _: middleware::logged_user::CheckUserCanAccessService,
    _: middleware::csrf_token::CsrfToken,
    session::WebAppSession { user, .. }: session::WebAppSession,
    app_state: web::types::State<AppState>,
    path: web::types::Path<(i64,)>,
) -> Result<impl web::Responder, web::Error> {
    let pet_id = path.0;
    api::pet::delete_pet_and_its_info(pet_id, user.id, &app_state.repo)
        .await
        .map_err(|e| errors::ServerError::InternalServerError(e.to_string()))?;

    Ok(web::HttpResponse::Ok()
        .set_header("HX-Trigger", "petRecordUpdated")
        .finish())
}

/// Generate QR code card for pet profile
#[web::get("qr_code/{pet_external_id}")]
async fn get_profile_qr_code(
    _: middleware::logged_user::CheckUserCanAccessService,
    path: web::types::Path<(Uuid,)>,
    app_state: web::types::State<AppState>,
) -> Result<impl web::Responder, web::Error> {
    let pet_external_id = path.0;
    let app_config = config::APP_CONFIG
        .get()
        .context("failed to get app config")
        .map_err(web::error::ErrorInternalServerError)?;
    let url = format!(
        "{base_url}/info/{external_id}",
        base_url = app_config.base_url(),
        external_id = pet_external_id
    );

    // Try to get pet picture
    let pet_pic =
        api::pet::get_public_pic(pet_external_id, &app_state.repo, &app_state.storage_service)
            .await
            .ok()
            .flatten();

    // Generate QR code card with picture if available, otherwise simple QR code
    let qr_code = if let Some(ref pic) = pet_pic {
        crate::qr::build_qr_card_with_pic(pic, &url)
    } else {
        crate::qr::get_qr_code(&url)
    }
    .map_err(|e| {
        errors::ServerError::InternalServerError(format!("qr_code could not be generated: {}", e))
    })?;

    let body = once(ok::<_, web::Error>(Bytes::from_iter(&qr_code)));

    Ok(web::HttpResponse::Ok()
        .content_type("image/png")
        .streaming(body))
}

/// Generate PDF report for pet
#[web::get("pdf_report/{pet_id}")]
async fn get_pdf_report(
    _: middleware::logged_user::CheckUserCanAccessService,
    path: web::types::Path<(i64,)>,
    session::WebAppSession { user, .. }: session::WebAppSession,
    app_state: web::types::State<AppState>,
) -> Result<impl web::Responder, web::Error> {
    let content = crate::api::pet::generate_pdf_report_bytes(
        path.0,
        user.id,
        &app_state.repo,
        &app_state.storage_service,
    )
    .await
    .map_err(|e| {
        errors::ServerError::InternalServerError(format!(
            "get_pdf_report could not generate the file: {e}"
        ))
    })?;

    let body = once(ok::<_, web::Error>(Bytes::from_iter(&content)));

    Ok(web::HttpResponse::Ok()
        .content_type("application/pdf")
        .streaming(body))
}

/// Serve pet's public profile picture
#[web::get("public_pic/{pet_external_id}")]
async fn get_pet_public_pic(
    path: web::types::Path<(Uuid,)>,
    app_state: web::types::State<AppState>,
) -> Result<impl web::Responder, web::Error> {
    let pet_external_id = path.0;
    let pet_pic =
        api::pet::get_public_pic(pet_external_id, &app_state.repo, &app_state.storage_service)
            .await
            .map_err(|e| {
                errors::ServerError::InternalServerError(format!(
                    "pet_public_pic could not be generated: {e}"
                ))
            })?;

    if let Some(pet_pic) = pet_pic {
        let body = once(ok::<_, web::Error>(Bytes::from_iter(&pet_pic.body)));

        return Ok(web::HttpResponse::Ok()
            .content_type(format!("image/{}", pet_pic.extension))
            .streaming(body));
    }

    Ok(web::HttpResponse::NoContent().into())
}

/// Serve webmanifest file
#[web::get("/site.webmanifest/{pet_external_id}")]
async fn serve_webmanifest(
    path: web::types::Path<(Uuid,)>,
) -> Result<impl web::Responder, web::Error> {
    let body = templates::WEB_MANIFESTS
        .render(
            "site.webmanifest",
            &tera::Context::from_value(json!({"external_id": path.0})).unwrap_or_default(),
        )
        .map_err(|e| {
            errors::ServerError::TemplateError(format!(
                "at /pet/site.webmanifest endpoint the template couldnt be rendered: {e}"
            ))
        })?;

    Ok(web::HttpResponse::Ok()
        .content_type("application/json; charset=utf-8")
        .body(body))
}

/// Generate Apple Wallet pass for pet
#[web::get("pass/{pet_external_id}")]
async fn download_pet_pass(
    _: middleware::logged_user::CheckUserCanAccessService,
    path: web::types::Path<(Uuid,)>,
    app_state: web::types::State<AppState>,
) -> Result<impl web::Responder, web::Error> {
    let pet_external_id = path.0;

    // Get pet public information
    let pet_info = api::pet::get_pet_public_info(pet_external_id, &app_state.repo)
        .await
        .map_err(|e| {
            errors::ServerError::InternalServerError(format!("Failed to get pet info: {e}"))
        })?;

    // Generate the pass
    let pass_data = api::passes::generate_pet_pass(&pet_info, &app_state.storage_service)
        .await
        .map_err(|e| {
            errors::ServerError::InternalServerError(format!("Failed to generate pass: {e}"))
        })?;

    let body = once(ok::<_, web::Error>(Bytes::from_iter(&pass_data)));

    Ok(web::HttpResponse::Ok()
        .content_type("application/vnd.apple.pkpass")
        .set_header(
            "Content-Disposition",
            "attachment; filename=\"pet_info.pkpass\"",
        )
        .streaming(body))
}

/// Render pet edit form with existing data
#[web::get("details/{pet_id}")]
async fn get_pet_details_form(
    _: middleware::logged_user::CheckUserCanAccessService,
    _: middleware::csrf_token::CsrfToken,
    session::WebAppSession { user, .. }: session::WebAppSession,
    app_state: web::types::State<AppState>,
    path: web::types::Path<(i64,)>,
) -> Result<impl web::Responder, web::Error> {
    let pet_id = path.0;
    let context = tera::Context::from_value(json!({
        "pet": api::pet::get_pet_user_to_edit(pet_id, user.id,&app_state.repo)
        .await
        .map_err(|e| {
            errors::ServerError::InternalServerError(format!(
                "at /pet/details/pet_id endpoint pet info [get_pet_user_to_edit] couldnt be retrieved: {e}"
            ))
        })?,
        "PIC_PET_MAX_SIZE_BYTES": consts::PIC_PET_MAX_SIZE_BYTES,
        "ACCEPTED_IMAGE_EXTENSIONS": consts::ACCEPTED_IMAGE_EXTENSIONS,
    }))
    .unwrap_or_default();

    let content = templates::WEB_TEMPLATES
        .render("pet_details.html", &context)
        .map_err(|e| {
            errors::ServerError::TemplateError(format!(
                "at /pet/details/pet_id endpoint the template couldnt be rendered: {e}"
            ))
        })?;

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(content))
}

/// Handle pet update form submission
#[web::post("details/{pet_id}")]
async fn edit_pet_details(
    _: middleware::logged_user::CheckUserCanAccessService,
    _: middleware::csrf_token::CsrfToken,
    session::WebAppSession { user, .. }: session::WebAppSession,
    app_state: web::types::State<AppState>,
    payload: ntex_multipart::Multipart,
    path: web::types::Path<(i64,)>,
) -> Result<impl web::Responder, web::Error> {
    let _span = logfire::span!("edit_pet_details").entered();

    let pet_form = forms::pet::CreatePetForm {
        id: path.0,
        ..deserialize_pet_form(payload)
            .await
            .map_err(|e| errors::UserError::FormInputValueError(e.to_string()))?
    };

    api::pet::update_pet_to_user(
        user.id,
        &user.email,
        pet_form,
        &app_state.repo,
        &app_state.storage_service,
    )
    .await
    .map_err(|e| errors::ServerError::InternalServerError(e.to_string()))?;

    utils::redirect_to("/pet")
}
