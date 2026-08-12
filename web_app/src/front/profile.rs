use crate::{
    api, consts,
    front::{AppState, errors, middleware, session, templates},
};
use ntex::web;
use ntex_identity::Identity;
use ntex_session::Session;
use tera::context;

/// Renders user profile app section
#[web::get("")]
async fn get_profile_view(
    session::WebAppSession { user, .. }: session::WebAppSession,
    app_state: web::types::State<AppState>,
) -> Result<impl web::Responder, web::Error> {
    let user_can_access_service = user.can_access_service();

    let context = context! {
        can_edit => &user_can_access_service,
        owner_contacts => &api::user::get_owner_contacts(user.id, None, &app_state.repo)
            .await
            .map_err(|e| {
                errors::ServerError::InternalServerError(format!(
                    "function get_owner_contacts raised an error: {e}"
                ))
            })?,
        payments => &api::user::get_payments(user.id, &app_state.repo)
        .await
        .map_err(|e| {
            errors::ServerError::InternalServerError(format!(
                "function get_payments raised an error: {e}"
            ))
        })?,
        otp_step => &(if user.phone_reminder.is_some() {"OTP_SUCCESS"} else {"OTP_START"}),
        phone_reminder => &user.phone_reminder,
        service_price => &format!("{:.2}", consts::ADD_PET_PRICE),
        can_access_service => &user_can_access_service,
        show_menu => &true,
    };

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

/// Handles the request to add a new contact item to a user
#[web::post("contact")]
async fn add_new_owner_contact(
    _: middleware::logged_user::CheckUserCanAccessService,
    session::WebAppSession { user, .. }: session::WebAppSession,
    form: web::types::Form<api::user::OwnerContactRequest>,
    app_state: web::types::State<AppState>,
    _: middleware::csrf_token::CsrfToken,
) -> Result<impl web::Responder, web::Error> {
    let form_request = api::user::OwnerContactRequest {
        contact_name: ammonia::clean(&form.contact_name),
        contact_value: ammonia::clean(&form.contact_value),
    };

    if !form_request.fields_are_valid() {
        return Ok(web::HttpResponse::BadRequest().finish());
    }

    api::user::add_owner_contact(user.id, &form_request, &app_state.repo)
        .await
        .map_err(|e| {
            errors::ServerError::InternalServerError(format!(
                "function add_owner_contact raised an error: {e}"
            ))
        })?;

    Ok(web::HttpResponse::Created()
        .set_header("HX-Trigger", "ownerContactRecordUpdated")
        .finish())
}

#[web::get("contact")]
async fn get_owner_contacts(
    session::WebAppSession { user, .. }: session::WebAppSession,
    app_state: web::types::State<AppState>,
) -> Result<impl web::Responder, web::Error> {
    let context = context! {
        can_edit => &user.can_access_service(),
        owner_contacts => &api::user::get_owner_contacts(user.id, None, &app_state.repo)
            .await
            .map_err(|e| {
                errors::ServerError::InternalServerError(format!(
                    "function get_owner_contacts raised an error: {e}"
                ))
            })?,
    };

    let content = templates::WEB_TEMPLATES
        .render("widgets/owner_contact_list.html", &context)
        .map_err(|e| {
            errors::ServerError::TemplateError(format!(
                "at /profile endpoint the template couldnt be rendered: {e}"
            ))
        })?;

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(content))
}

#[web::delete("contact/{contact_id}")]
async fn delete_owner_contact(
    _: middleware::logged_user::CheckUserCanAccessService,
    session::WebAppSession { user, .. }: session::WebAppSession,
    path: web::types::Path<(i64,)>,
    app_state: web::types::State<AppState>,
    _: middleware::csrf_token::CsrfToken,
) -> Result<impl web::Responder, web::Error> {
    let contact_id = path.0;

    api::user::delete_owner_contact(user.id, contact_id, &app_state.repo)
        .await
        .map_err(|e| {
            errors::ServerError::WidgetTemplateError(format!(
                "function delete_owner_contact raised an error: {e}"
            ))
        })?;

    Ok(web::HttpResponse::Ok()
        .set_header("HX-Trigger", "ownerContactRecordUpdated")
        .finish())
}

/// Deletes all data filled by the user app
#[web::post("/delete-data")]
async fn delete_user_data(
    session::WebAppSession { user, .. }: session::WebAppSession,
    app_state: web::types::State<AppState>,
    identity: Identity,
    _: middleware::csrf_token::CsrfToken,
) -> Result<impl web::Responder, web::Error> {
    api::user::delete_user_data(user.id, &app_state.repo)
        .await
        .map_err(|e| {
            errors::ServerError::WidgetTemplateError(format!(
                "function delete_owner_contact raised an error: {e}"
            ))
        })?;

    identity.forget();

    let content = templates::WEB_TEMPLATES
        .render("account_data_deleted.html", &tera::Context::new())
        .map_err(|e| {
            errors::ServerError::TemplateError(format!(
                "account_data_deleted endpoint the template couldnt be rendered: {e}"
            ))
        })?;

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(content))
}

#[web::delete("/close-session")]
async fn close_session(
    cookie: Session,
    identity: Identity,
) -> Result<impl web::Responder, web::Error> {
    identity.forget();
    cookie.clear();

    Ok(web::HttpResponse::Ok()
        .set_header("HX-Redirect", "/")
        .finish())
}
