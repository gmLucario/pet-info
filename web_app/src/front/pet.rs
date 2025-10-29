//! Pet management handlers for the web frontend
//!
//! This module contains all HTTP handlers related to pet management functionality,
//! including CRUD operations, file uploads, PDF generation, and Apple Wallet pass creation.
//!
//! # Routes Overview
//! - `GET /pet` - Pet dashboard showing all user's pets
//! - `GET /pet/list` - HTMX endpoint for pets list widget
//! - `GET /pet/new` - Form for creating new pets
//! - `POST /pet/new` - Handle pet creation
//! - `DELETE /pet/delete/{pet_id}` - Delete a pet
//! - `GET /pet/details/{pet_id}` - Form for editing pet details
//! - `POST /pet/details/{pet_id}` - Handle pet updates
//! - `GET /pet/qr_code/{pet_external_id}` - Generate QR code for pet profile
//! - `GET /pet/pdf_report/{pet_id}` - Generate PDF report
//! - `GET /pet/public_pic/{pet_external_id}` - Serve public pet pictures
//! - `GET /pet/pass/{pet_external_id}` - Generate Apple Wallet pass
//!
//! # Security
//! Most routes require authentication and user ownership validation.
//! CSRF protection is enabled for state-changing operations.

use anyhow::{Context, bail};
use chrono::NaiveDateTime;
use futures::{TryStreamExt, future::ok, stream::once};
use ntex::{util::Bytes, web};
use serde_json::json;
use std::{path::Path, str::FromStr};
use uuid::Uuid;

use crate::{
    api, config, consts,
    front::{AppState, errors, forms, middleware, session, templates, utils},
    models::pet::PetNote,
};

/// Safely extracts header value as string from HTTP headers
///
/// # Arguments
/// * `headers` - HTTP headers map
/// * `key` - Header key to extract
///
/// # Returns
/// String value of the header or empty string if not found/invalid
fn get_header_str_value(headers: &ntex::http::HeaderMap, key: &str) -> String {
    let default_header_value = ntex::http::header::HeaderValue::from_static("");

    headers
        .get(key)
        .unwrap_or(&default_header_value)
        .to_str()
        .unwrap_or_default()
        .to_string()
}

/// Extracts file extension from Content-Disposition header
///
/// # Arguments
/// * `content_disposition` - Content-Disposition header value
///
/// # Returns
/// * `Ok(String)` - File extension in lowercase
/// * `Err(anyhow::Error)` - If extension cannot be parsed
///
/// # Example
/// ```
/// let header = "form-data; name=\"file\"; filename=\"image.jpg\"";
/// assert_eq!(get_filename_extension(header)?, "jpg");
/// ```
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

/// Checks if the field contains an image for pet picture upload
fn is_image_field(field: &ntex_multipart::Field, content_disposition: &str) -> bool {
    field.content_type().essence_str().contains("image") && content_disposition.contains("pet_pic")
}

/// Processes an image field, validating size and extracting file data
async fn process_image_field(
    field: ntex_multipart::Field,
    content_disposition: &str,
) -> anyhow::Result<forms::pet::Pic> {
    let body = utils::get_bytes_value(field).await;

    // Validate image size
    if body.len() > consts::PIC_PET_MAX_SIZE_BYTES {
        bail!(
            "Image is too big. Maximum size: {} bytes",
            consts::PIC_PET_MAX_SIZE_BYTES
        );
    }

    Ok(forms::pet::Pic {
        filename_extension: get_filename_extension(content_disposition)?,
        body,
    })
}

/// Deserializes multipart form data into a pet creation form
///
/// Handles both text fields and image uploads with cropping support.
/// Validates image size and applies circular cropping if cropper data is provided.
///
/// # Arguments
/// * `payload` - Multipart form data from HTTP request
///
/// # Returns
/// * `Ok(CreatePetForm)` - Parsed and validated form data
/// * `Err(anyhow::Error)` - If parsing fails or image is too large
///
/// # Image Processing
/// - Validates image size against `PIC_PET_MAX_SIZE_BYTES` limit
/// - Applies circular cropping if cropper coordinates are provided
/// - Sanitizes all text inputs with ammonia
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
            pet_pic = Some(process_image_field(field, &content_disposition).await?);
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

/// Main pet dashboard page displaying all user's pets
///
/// Renders the complete pet management interface with:
/// - List of all user's pets with basic information
/// - Pet balance information
/// - Navigation to pet creation and editing
///
/// # Authentication
/// Requires valid user session
///
/// # Returns
/// * `Ok(HttpResponse)` - Rendered HTML page with pets list
/// * `Err(web::Error)` - Server error if database operation fails
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

/// HTMX endpoint returning pets list widget HTML
///
/// Returns only the pets list portion for dynamic updates.
/// Used by HTMX to refresh the pets display without full page reload.
///
/// # Authentication
/// Requires valid user session
///
/// # Returns
/// * `Ok(HttpResponse)` - HTML widget with pets list
/// * `Err(web::Error)` - Server error if database operation fails
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

/// Query parameters for pet form rendering
#[derive(serde::Deserialize, Debug)]
struct PetFormQueryParams {
    /// Optional external pet ID for pre-linking with physical pet tags
    pet_external_id: Option<Uuid>,
}

/// Renders the pet creation form
///
/// Displays an empty form for creating new pets or a form pre-filled
/// with external pet ID if provided via query parameters.
///
/// # Query Parameters
/// * `pet_external_id` - Optional UUID for linking with physical pet tags
///
/// # Returns
/// * `Ok(HttpResponse)` - Rendered pet creation form
/// * `Err(web::Error)` - Template rendering error
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

/// Handles pet creation form submission
///
/// Creates a new pet if the user has sufficient balance or is linking
/// an existing pet via external ID. Updates user session with new balance.
///
/// # Security
/// - Requires CSRF token
/// - Validates user subscription and pet balance
/// - Sanitizes all form inputs
///
/// # Form Processing
/// - Handles multipart form with image upload
/// - Applies image cropping if provided
/// - Uploads image to S3 storage
/// - Updates user session state
///
/// # Returns
/// * `Ok(Redirect)` - Redirects to pet dashboard on success
/// * `Err(web::Error)` - User error for invalid input or server error
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

/// Deletes a pet and all associated data
///
/// Removes the pet and cascades deletion to all related records:
/// - Health records (vaccines, deworms)
/// - Weight measurements
/// - Notes
/// - External ID linkage
///
/// # Security
/// - Requires service access (subscription)
/// - Requires CSRF token
/// - Validates user ownership
///
/// # Returns
/// * `Ok(HttpResponse)` - Success response with HTMX trigger
/// * `Err(web::Error)` - Server error if deletion fails
///
/// # Note
/// Does not restore pet balance - deletion is permanent
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

/// Generates and streams QR code for pet's public profile
///
/// Creates a QR code containing the URL to the pet's public information page.
/// The QR code can be printed on physical pet tags for easy scanning.
///
/// # Path Parameters
/// * `pet_external_id` - UUID of the pet's external identifier
///
/// # Security
/// Requires service access (subscription)
///
/// # Returns
/// * `Ok(HttpResponse)` - JPEG image stream of the QR code
/// * `Err(web::Error)` - Server error if QR generation fails
///
/// # Generated URL Format
/// `{base_url}/info/{external_id}`
#[web::get("qr_code/{pet_external_id}")]
async fn get_profile_qr_code(
    _: middleware::logged_user::CheckUserCanAccessService,
    path: web::types::Path<(Uuid,)>,
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
    let qr_code = super::utils::get_qr_code(&url).map_err(|e| {
        errors::ServerError::InternalServerError(format!("qr_code could not be generated: {}", e))
    })?;

    let body = once(ok::<_, web::Error>(Bytes::from_iter(&qr_code)));

    Ok(web::HttpResponse::Ok()
        .content_type("image/jpeg")
        .streaming(body))
}

/// Weight measurement data for PDF report generation
#[derive(Debug, Clone, Default, serde::Serialize, PartialEq)]
struct WeightReport {
    /// Weight value in user's preferred units
    pub value: f64,
    /// Timestamp when the weight was recorded
    pub created_at: NaiveDateTime,
    /// Formatted age string (e.g., "3 months, 2 weeks")
    pub fmt_age: String,
}

/// Converts HTML content in pet notes to plain text
fn convert_html_to_text(note: &PetNote) -> PetNote {
    PetNote {
        content: html2text::from_read(note.content.as_bytes(), 20)
            .unwrap_or(note.content.to_string()),
        ..note.clone()
    }
}

/// Generates and streams a comprehensive PDF report for a pet
///
/// Creates a formatted PDF document containing:
/// - Pet basic information (name, breed, age, etc.)
/// - Complete health records (vaccines, deworms)
/// - Weight history with age calculations
/// - Pet notes
/// - QR code link to public profile
///
/// # Path Parameters
/// * `pet_id` - Internal database ID of the pet
///
/// # Security
/// - Requires service access (subscription)
/// - Validates user ownership of the pet
///
/// # Returns
/// * `Ok(HttpResponse)` - PDF document stream
/// * `Err(web::Error)` - Server error if PDF generation fails
///
/// # Template
/// Uses Typst template engine for professional formatting
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
                "notes": pet_full_info.notes.iter().map(convert_html_to_text).collect::<Vec<_>>(),
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

/// Serves the pet's public profile picture
///
/// Returns the pet's image from S3 storage for display on public profiles.
/// No authentication required - this endpoint serves public pet information.
///
/// # Path Parameters
/// * `pet_external_id` - UUID of the pet's external identifier
///
/// # Returns
/// * `Ok(HttpResponse)` - Image stream with appropriate content-type
/// * `Ok(NoContent)` - If no image is available for the pet
/// * `Err(web::Error)` - Server error if storage access fails
///
/// # Content Types
/// Supports various image formats based on stored file extension
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

/// Serve `webmanifest`
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

/// Generates and downloads Apple Wallet pass for pet information
///
/// Creates a .pkpass file that can be added to iOS Wallet containing:
/// - Pet name and basic information
/// - QR code for quick profile access
/// - Emergency contact information
/// - Professional styling with pet photo
///
/// # Path Parameters
/// * `pet_external_id` - UUID of the pet's external identifier
///
/// # Security
/// Requires service access (subscription)
///
/// # Returns
/// * `Ok(HttpResponse)` - .pkpass file download
/// * `Err(web::Error)` - Server error if pass generation fails
///
/// # Apple Wallet Integration
/// The generated pass follows Apple's PKPass format specification
/// and includes proper signing for iOS compatibility
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

/// Renders the pet editing form with existing data
///
/// Displays a form pre-populated with the pet's current information
/// for editing. All fields are editable including image upload.
///
/// # Path Parameters
/// * `pet_id` - Internal database ID of the pet to edit
///
/// # Security
/// - Requires service access (subscription)
/// - Requires CSRF token
/// - Validates user ownership of the pet
///
/// # Returns
/// * `Ok(HttpResponse)` - Rendered edit form with current pet data
/// * `Err(web::Error)` - Server error if pet retrieval fails
///
/// # Features
/// - Pre-fills all form fields with existing pet data
/// - Supports image upload and cropping
/// - Input validation and sanitization
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

/// Processes pet information update form submission
///
/// Updates an existing pet's information including:
/// - Basic details (name, breed, birthday, etc.)
/// - Behavioral flags (lost status, spay/neuter)
/// - Pet description
/// - Profile picture with cropping support
///
/// # Path Parameters
/// * `pet_id` - Internal database ID of the pet to update
///
/// # Security
/// - Requires service access (subscription)
/// - Requires CSRF token
/// - Validates user ownership of the pet
/// - Sanitizes all form inputs
///
/// # Form Processing
/// - Handles multipart form with optional image upload
/// - Applies image cropping if coordinates provided
/// - Updates S3 storage with new images
/// - Preserves existing data for unchanged fields
///
/// # Returns
/// * `Ok(Redirect)` - Redirects to pet dashboard on success
/// * `Err(web::Error)` - User error for invalid input or server error
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
