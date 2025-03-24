use crate::{
    api, consts,
    front::{
        errors,
        middleware::{self, logged_user::IsUserLoggedAndCanEdit},
        templates, AppState,
    },
    models,
};
use ntex::web;
use ntex_identity::Identity;
use serde_json::json;

#[web::get("")]
async fn get_profile_view(
    logged_user: models::user_app::User,
    IsUserLoggedAndCanEdit(can_edit, _): IsUserLoggedAndCanEdit,
    app_state: web::types::State<AppState>,
) -> Result<impl web::Responder, web::Error> {
    let context = tera::Context::from_value(json!({
        "can_edit": &can_edit,
        "owner_contacts": &api::user::get_owner_contacts(logged_user.id, None, &app_state.repo)
            .await
            .map_err(|e| {
                errors::ServerError::InternalServerError(format!(
                    "function get_owner_contacts raised an error: {e}"
                ))
            })?,
        "payments": &api::user::get_payments(logged_user.id, &app_state.repo)
        .await
        .map_err(|e| {
            errors::ServerError::InternalServerError(format!(
                "function get_payments raised an error: {e}"
            ))
        })?,
        "otp_step": if logged_user.phone_reminder.is_some() {"OTP_SUCCESS"} else {"OTP_START"},
        "phone_reminder": logged_user.phone_reminder,
        "service_price": &format!("{:.2}", consts::SERVICE_PRICE),
        "end_free_trial": &logged_user.end_free_trial(),
    }))
    .unwrap_or_default();

    let content = templates::WEB_TEMPLATES
        .render("profile.html", &context)
        .map_err(|e| {
            errors::ServerError::TemplateError(format!(
                "at /profile endpoint the template couldnt be rendered: {e}"
            ))
        })?;

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(content))
}

#[web::post("contact")]
async fn add_new_owner_contact(
    _: middleware::logged_user::CheckUserCanAccessService,
    logged_user: models::user_app::User,
    form: web::types::Form<api::user::OwnerContactRequest>,
    app_state: web::types::State<AppState>,
    _: middleware::csrf_token::CsrfToken,
) -> Result<impl web::Responder, web::Error> {
    let form_request = api::user::OwnerContactRequest {
        contact_name: ammonia::clean(&form.0.contact_name),
        contact_value: ammonia::clean(&form.0.contact_value),
    };

    api::user::add_owner_contact(logged_user.id, &form_request, &app_state.repo)
        .await
        .map_err(|e| {
            errors::ServerError::InternalServerError(format!(
                "function add_owner_contact raised an error: {}",
                e
            ))
        })?;

    Ok(web::HttpResponse::Created()
        .set_header("HX-Trigger", "ownerContactRecordUpdated")
        .body(""))
}

#[web::get("contact")]
async fn get_owner_contacts(
    IsUserLoggedAndCanEdit(can_edit, user_id): IsUserLoggedAndCanEdit,
    app_state: web::types::State<AppState>,
) -> Result<impl web::Responder, web::Error> {
    let mut context = tera::Context::new();
    context.insert("can_edit", &can_edit);

    if let Some(user_id) = user_id {
        context.insert(
            "owner_contacts",
            &api::user::get_owner_contacts(user_id, None, &app_state.repo)
                .await
                .map_err(|e| {
                    errors::ServerError::InternalServerError(format!(
                        "function get_owner_contacts raised an error: {}",
                        e
                    ))
                })?,
        );
    }

    let content = templates::WEB_TEMPLATES
        .render("widgets/owner_contact_list.html", &context)
        .map_err(|e| {
            errors::ServerError::TemplateError(format!(
                "at /profile endpoint the template couldnt be rendered: {}",
                e
            ))
        })?;

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(content))
}

#[web::delete("contact/{contact_id}")]
async fn delete_owner_contact(
    _: middleware::logged_user::CheckUserCanAccessService,
    logged_user: models::user_app::User,
    path: web::types::Path<(i64,)>,
    app_state: web::types::State<AppState>,
    _: middleware::csrf_token::CsrfToken,
) -> Result<impl web::Responder, web::Error> {
    let contact_id = path.0;

    api::user::delete_owner_contact(logged_user.id, contact_id, &app_state.repo)
        .await
        .map_err(|e| {
            errors::ServerError::WidgetTemplateError(format!(
                "function delete_owner_contact raised an error: {}",
                e
            ))
        })?;

    Ok(web::HttpResponse::Ok()
        .set_header("HX-Trigger", "ownerContactRecordUpdated")
        .body(""))
}

#[web::post("/delete-data")]
async fn delete_user_data(
    logged_user: models::user_app::User,
    app_state: web::types::State<AppState>,
    identity: Identity,
    _: middleware::csrf_token::CsrfToken,
) -> Result<impl web::Responder, web::Error> {
    api::user::delete_user_data(logged_user.id, &app_state.repo)
        .await
        .map_err(|e| {
            errors::ServerError::WidgetTemplateError(format!(
                "function delete_owner_contact raised an error: {}",
                e
            ))
        })?;

    identity.forget();

    let content = templates::WEB_TEMPLATES
        .render("account_data_deleted.html", &tera::Context::new())
        .map_err(|e| {
            errors::ServerError::TemplateError(format!(
                "account_data_deleted endpoint the template couldnt be rendered: {}",
                e
            ))
        })?;

    Ok(web::HttpResponse::Ok().body(content))
}
