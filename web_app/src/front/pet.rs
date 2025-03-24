use std::path::Path;

use anyhow::bail;
use futures::{future::ok, stream::once, TryStreamExt};
use ntex::{util::Bytes, web};
use serde_json::json;
use uuid::Uuid;

use crate::{
    api, consts,
    front::{errors, forms, middleware, templates, utils, AppState},
    models,
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

    if let Some(filename) = Path::new(sections.next().unwrap_or_default()).extension() {
        if let Some(filename) = filename.to_str() {
            return Ok(filename.to_string().trim().to_lowercase());
        }
    }

    bail!("filename extension couldnt be found in the request content_disposition form")
}

async fn deserialize_pet_form(
    mut payload: ntex_multipart::Multipart,
) -> anyhow::Result<super::forms::pet::CreatePetForm> {
    let mut form = super::forms::pet::CreatePetForm::default();

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

            form.pet_pic = Some(super::forms::pet::PetPic {
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
        }
    }

    Ok(form)
}

#[web::get("")]
async fn get_pet_view(
    _: middleware::logged_user::CheckUserCanAccessService,
    logged_user: models::user_app::User,
    app_state: web::types::State<AppState>,
) -> Result<impl web::Responder, web::Error> {
    let context = tera::Context::from_value(json!({
        "pets": api::pet::get_user_pets_cards(logged_user.id, &app_state.repo)
        .await
        .map_err(|e| {
            errors::ServerError::InternalServerError(format!(
                "function get_user_pets_cards raised an error: {}",
                e
            ))
        })?,
    }))
    .unwrap_or_default();

    let content = templates::WEB_TEMPLATES
        .render("pet.html", &context)
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

#[web::get("/list")]
async fn user_pets_list(
    _: middleware::logged_user::CheckUserCanAccessService,
    logged_user: models::user_app::User,
    app_state: web::types::State<AppState>,
) -> Result<impl web::Responder, web::Error> {
    let context = tera::Context::from_value(json!({
        "pets": api::pet::get_user_pets_cards(logged_user.id, &app_state.repo)
        .await
        .map_err(|e| {
            errors::ServerError::InternalServerError(format!(
                "function get_user_pets_cards raised an error: {}",
                e
            ))
        })?,
    }))
    .unwrap_or_default();

    let content = templates::WEB_TEMPLATES
        .render("widgets/pets.html", &context)
        .map_err(|e| {
            errors::ServerError::TemplateError(format!(
                "at /pet/list endpoint the template couldnt be rendered: {}",
                e
            ))
        })?;

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(content))
}

#[web::get("/new")]
async fn render_pet_details_form(
    _: middleware::logged_user::CheckUserCanAccessService,
) -> Result<impl web::Responder, web::Error> {
    let content = templates::WEB_TEMPLATES
        .render(
            "pet_details.html",
            &tera::Context::from_value(json!({
                "PIC_PET_MAX_SIZE_BYTES": consts::PIC_PET_MAX_SIZE_BYTES,
                "ACCEPTED_IMAGE_EXTENSIONS": consts::ACCEPTED_IMAGE_EXTENSIONS,
            }))
            .unwrap_or_default(),
        )
        .map_err(|e| {
            errors::ServerError::TemplateError(format!(
                "at /pet/new endpoint the template couldnt be rendered: {}",
                e
            ))
        })?;

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(content))
}

#[web::post("/new")]
async fn create_pet_request(
    _: middleware::logged_user::CheckUserCanAccessService,
    _: middleware::csrf_token::CsrfToken,
    logged_user: models::user_app::User,
    app_state: web::types::State<AppState>,
    payload: ntex_multipart::Multipart,
) -> Result<impl web::Responder, web::Error> {
    let pet_form = deserialize_pet_form(payload)
        .await
        .map_err(|e| errors::UserError::FormInputValueError(e.to_string()))?;

    api::pet::add_new_pet_to_user(
        logged_user.id,
        &logged_user.email,
        pet_form,
        &app_state.repo,
        &app_state.storage_service,
    )
    .await
    .map_err(|e| errors::ServerError::InternalServerError(e.to_string()))?;

    utils::redirect_to("/pet")
}

#[web::delete("/delete/{pet_id}")]
async fn delete_pet(
    _: middleware::logged_user::CheckUserCanAccessService,
    _: middleware::csrf_token::CsrfToken,
    logged_user: models::user_app::User,
    app_state: web::types::State<AppState>,
    path: web::types::Path<(i64,)>,
) -> Result<impl web::Responder, web::Error> {
    let pet_id = path.0;
    api::pet::delete_pet_and_its_info(pet_id, logged_user.id, &app_state.repo)
        .await
        .map_err(|e| errors::ServerError::InternalServerError(e.to_string()))?;

    Ok(web::HttpResponse::Ok()
        .set_header("HX-Trigger", "petRecordUpdated")
        .body(""))
}

#[web::get("info/{pet_external_id}")]
async fn get_pet_info_view(
    app_state: web::types::State<AppState>,
    path: web::types::Path<(Uuid,)>,
) -> Result<impl web::Responder, web::Error> {
    let pet_external_id = path.0;
    let pet = api::pet::get_pet_public_info(pet_external_id, &app_state.repo)
        .await
        .map_err(|e| {
            errors::ServerError::InternalServerError(format!(
                "at /pet/external_id endpoint pet info couldnt be retrieved: {}",
                e
            ))
        })?;

    let context = tera::Context::from_value(json!({
        "pet": pet,
        "owner_contacts": api::user::get_owner_contacts(0, Some(pet_external_id), &app_state.repo)
        .await
        .map_err(|e| {
            errors::ServerError::InternalServerError(format!(
                "function get_owner_contacts raised an error: {}",
                e
            ))
        })?
    }))
    .unwrap_or_default();

    let content = templates::WEB_TEMPLATES
        .render("pet_public_info.html", &context)
        .map_err(|e| {
            errors::ServerError::TemplateError(format!(
                "at /pet/external_id endpoint the template couldnt be rendered: {}",
                e
            ))
        })?;

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(content))
}

#[web::get("qr_code/{pet_external_id}")]
async fn get_profile_qr_code(
    _: middleware::logged_user::CheckUserCanAccessService,
    path: web::types::Path<(Uuid,)>,
) -> Result<impl web::Responder, web::Error> {
    let pet_external_id = path.0;
    let qr_code = super::utils::get_qr_code(format!("https://pet-info.in/pet/{}", pet_external_id))
        .map_err(|e| {
            errors::ServerError::InternalServerError(format!(
                "qr_code could not be generated: {}",
                e
            ))
        })?;

    let body = once(ok::<_, web::Error>(Bytes::from_iter(&qr_code)));

    Ok(web::HttpResponse::Ok()
        .content_type("image/jpeg")
        .streaming(body))
}

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
                    "pet_public_pic could not be generated: {}",
                    e
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

#[web::get("details/{pet_id}")]
async fn get_pet_details_form(
    _: middleware::logged_user::CheckUserCanAccessService,
    _: middleware::csrf_token::CsrfToken,
    logged_user: models::user_app::User,
    app_state: web::types::State<AppState>,
    path: web::types::Path<(i64,)>,
) -> Result<impl web::Responder, web::Error> {
    let pet_id = path.0;
    let context = tera::Context::from_value(json!({
        "pet": api::pet::get_pet_user_to_edit(pet_id, logged_user.id,&app_state.repo)
        .await
        .map_err(|e| {
            errors::ServerError::InternalServerError(format!(
                "at /pet/details/pet_id endpoint pet info [get_pet_user_to_edit] couldnt be retrieved: {}",
                e
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
                "at /pet/details/pet_id endpoint the template couldnt be rendered: {}",
                e
            ))
        })?;

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(content))
}

#[web::post("details/{pet_id}")]
async fn edit_pet_details(
    _: middleware::logged_user::CheckUserCanAccessService,
    _: middleware::csrf_token::CsrfToken,
    logged_user: models::user_app::User,
    app_state: web::types::State<AppState>,
    payload: ntex_multipart::Multipart,
    path: web::types::Path<(i64,)>,
) -> Result<impl web::Responder, web::Error> {
    let pet_form = forms::pet::CreatePetForm {
        id: path.0,
        ..deserialize_pet_form(payload)
            .await
            .map_err(|e| errors::UserError::FormInputValueError(e.to_string()))?
    };

    api::pet::update_pet_to_user(
        logged_user.id,
        &logged_user.email,
        pet_form,
        &app_state.repo,
        &app_state.storage_service,
    )
    .await
    .map_err(|e| errors::ServerError::InternalServerError(e.to_string()))?;

    utils::redirect_to("/pet")
}
