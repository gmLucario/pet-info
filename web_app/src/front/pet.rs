//! Handlers related to the /pet url

use anyhow::bail;
use chrono::NaiveDateTime;
use futures::{TryStreamExt, future::ok, stream::once};
use ntex::{util::Bytes, web};
use serde_json::json;
use std::{path::Path, str::FromStr};
use uuid::Uuid;

use crate::{
    api,
    config::APP_CONFIG,
    consts,
    front::{AppState, errors, forms, middleware, session, templates, utils},
};

fn get_header_str_value(headers: &ntex::http::HeaderMap, key: &str) -> String {
    let default_header_value = ntex::http::header::HeaderValue::from_static("");

    headers
        .get(key)
        .unwrap_or(&default_header_value)
        .to_str()
        .unwrap_or_default()
        .to_string()
}

fn get_filename_extension(content_disposition: &str) -> anyhow::Result<String> {
    let sections = content_disposition.split(";").collect::<Vec<&str>>();
    let mut sections = sections
        .iter()
        .filter(|s| s.trim().starts_with("filename="))
        .map(|w| {
            let name = &w.trim()["filename=".len()..];
            name.trim_matches('"')
        });

    if let Some(Some(filename)) = Path::new(sections.next().unwrap_or_default())
        .extension()
        .map(|filename| {
            filename
                .to_str()
                .map(|f| f.to_string().trim().to_lowercase())
        })
    {
        return Ok(filename);
    }

    bail!("filename extension couldnt be found in the request content_disposition form")
}

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

        if field.content_type().essence_str().contains("image")
            && content_disposition.contains("pet_pic")
        {
            let body = utils::get_bytes_value(field).await;
            let body_size = body.len();

            if body_size > consts::PIC_PET_MAX_SIZE_BYTES {
                bail!(
                    "image is to big. max size: {}",
                    consts::PIC_PET_MAX_SIZE_BYTES
                )
            }

            pet_pic = Some(forms::pet::Pic {
                filename_extension: get_filename_extension(&content_disposition)?,
                body,
            });

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
            filename_extension: pet_pic.filename_extension.to_string(),
            body: utils::crop_circle(&pet_pic, cropper_box.x, cropper_box.y, cropper_box.diameter)?,
        })
    }

    Ok(form)
}

/// Renders the pets view section:
/// List all user's pets
/// Start PetInfo id tag payment flow
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

/// Renders all user's pets
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

/// Renders the form of a specific pet, filled previously by its owner
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

/// Handles the request to create a new pet
/// A new pet can be created if the user has a positive pet balance
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

/// Handles the request to delete a user's pet
/// The user has to pay or acquire a Pet Info id tag previously
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

/// Stream the qr pet info link public profile
/// This handler builds the qr pet request
#[web::get("qr_code/{pet_external_id}")]
async fn get_profile_qr_code(
    _: middleware::logged_user::CheckUserCanAccessService,
    path: web::types::Path<(Uuid,)>,
) -> Result<impl web::Responder, web::Error> {
    let pet_external_id = path.0;
    let url = format!(
        "{base_url}/info/{external_id}",
        base_url = APP_CONFIG.base_url(),
        external_id = pet_external_id
    );
    let qr_code = super::utils::get_qr_code(&url).map_err(|e| {
        errors::ServerError::InternalServerError(format!("qr_code could not be generated: {}", e))
    })?;

    let body = once(ok::<_, web::Error>(Bytes::from_iter(&qr_code)));

    Ok(web::HttpResponse::Ok()
        .content_type("image/jpeg")
        .streaming(body))
}

#[derive(Debug, Clone, Default, serde::Serialize, PartialEq)]
struct WeightReport {
    pub value: f64,
    pub created_at: NaiveDateTime,
    pub fmt_age: String,
}

/// Builds and serve the pet's pdf report per each call
/// received, based on the `pet_id`
#[web::get("pdf_report/{pet_id}")]
async fn get_pdf_report(
    _: middleware::logged_user::CheckUserCanAccessService,
    path: web::types::Path<(i64,)>,
    session::WebAppSession { user, .. }: session::WebAppSession,
    app_state: web::types::State<AppState>,
) -> Result<impl web::Responder, web::Error> {
    let pet_full_info = api::pet::get_full_info(path.0, user.id, &app_state.repo)
        .await
        .map_err(|e| {
            errors::ServerError::InternalServerError(format!(
                "get_full_info could not be retrieved: {e}"
            ))
        })?;

    let now = utils::get_utc_now_with_default_time().date_naive();

    let content = templates::PDF_REPORT_TEMPLATES
        .render(
            "pet_default.typ",
            &tera::Context::from_value(json!({
                "pet_name": pet_full_info.pet.pet_name,
                "birthday": pet_full_info.pet.birthday,
                "age": utils::fmt_dates_difference(pet_full_info.pet.birthday, now),
                "breed": pet_full_info.pet.breed,
                "is_female": pet_full_info.pet.is_female,
                "is_spaying_neutering": pet_full_info.pet.is_spaying_neutering,
                "pet_link": format!(
                    "https://pet-info.link/info/{external_id}",
                    external_id=pet_full_info.pet.external_id
                ),
                "vaccines": pet_full_info.vaccines,
                "deworms": pet_full_info.deworms,
                "weights": pet_full_info.weights.iter().map(|w| WeightReport{
                    value: w.value,
                    created_at: w.created_at,
                    fmt_age: utils::fmt_dates_difference(pet_full_info.pet.birthday, w.created_at.into()),
                }).collect::<Vec<WeightReport>>(),
                "notes": pet_full_info.notes,
            }))
            .unwrap_or_default(),
        )
        .map_err(|e| {
            errors::ServerError::TemplateError(format!(
                "at /blog endpoint the template couldnt be rendered: {e}"
            ))
        })?;

    let content = api::pdf_handler::create_pdf_bytes_from_str(&content).map_err(|e| {
        errors::ServerError::TemplateError(format!(
            "at /blog endpoint the template couldnt be rendered: {e}"
        ))
    })?;

    let body = once(ok::<_, web::Error>(Bytes::from_iter(&content)));

    Ok(web::HttpResponse::Ok()
        .content_type("application/pdf")
        .streaming(body))
}

/// Serves the public pet image previously saved by its user
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

/// Downloads Apple Wallet pass for a pet's public information.
///
/// Generates and serves a .pkpass file that can be added to iOS Wallet.
/// The pass contains essential pet information for quick access.
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

/// Renders the form to edit a pet info data. If the field was
/// filled previously, the form will be populated with that
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

/// Handles the request from a pet form to edit a pet info data.
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
