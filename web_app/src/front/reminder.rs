use crate::{
    api, consts,
    front::{
        AppState, errors, forms,
        middleware::{
            csrf_token::CsrfToken,
            logged_user::{CheckUserCanAccessService, IsUserLoggedAndCanEdit},
        },
        session, templates, utils,
    },
};
use chrono_tz::Tz;
use ntex::web;
use ntex_identity::Identity;
use ntex_session::Session;
use serde_json::json;

/// Renders the reminder view section
#[web::get("")]
async fn get_reminder_view(
    _: IsUserLoggedAndCanEdit,
    session::WebAppSession { user, .. }: session::WebAppSession,
    app_state: web::types::State<AppState>,
) -> Result<impl web::Responder, web::Error> {
    let context = tera::Context::from_value(json!({
        "reminders": api::reminder::get_scheduled_reminders(user.id, &app_state.repo).await.unwrap_or_default(),
        "can_schedule_reminder": user.phone_reminder.is_some(),
    })).unwrap_or_default();

    let content = templates::WEB_TEMPLATES
        .render("reminders.html", &context)
        .map_err(|e| {
            errors::ServerError::TemplateError(format!(
                "at /reminder endpoint the template couldnt be rendered: {e}"
            ))
        })?;

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(content))
}

/// Retrieves the remider items to fill
/// the reminders table view
#[web::get("/tbody")]
async fn get_reminder_records(
    _: IsUserLoggedAndCanEdit,
    session::WebAppSession { user, .. }: session::WebAppSession,
    app_state: web::types::State<AppState>,
) -> Result<impl web::Responder, web::Error> {
    let context = tera::Context::from_value(json!({
        "reminders": api::reminder::get_scheduled_reminders(user.id, &app_state.repo).await.unwrap_or_default(),
    })).unwrap_or_default();

    let content = templates::WEB_TEMPLATES
        .render("widgets/tbody_reminder.html", &context)
        .map_err(|e| {
            errors::ServerError::WidgetTemplateError(format!(
                "at /reminder/tbody endpoint the template couldnt be rendered: {e}"
            ))
        })?;

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(content))
}

#[web::get("/send-verification-code")]
async fn start_verification_code_to_reminder_phone(
    _: CheckUserCanAccessService,
) -> Result<impl web::Responder, web::Error> {
    let context = tera::Context::from_value(json!({
        "otp_step": "OTP_START",
    }))
    .unwrap_or_default();

    let content = templates::WEB_TEMPLATES
        .render("widgets/otp.html", &context)
        .unwrap_or("Intente mas tarde".to_string());

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(content))
}

#[web::post("/send-verification-code")]
async fn send_verification_code_to_reminder_phone(
    _: CheckUserCanAccessService,
    form: web::types::Form<forms::user::ReminderPhoneToVerify>,
    cookie: Session,
    _: CsrfToken,
) -> Result<impl web::Responder, web::Error> {
    let context = tera::Context::from_value(json!({
        "otp_step": "OTP_VERIFICATION",
    }))
    .unwrap_or_default();

    let phone_number = format!(
        "{country_code}{phone}",
        country_code = form.country_phone_code,
        phone = form.reminders_phone
    );

    api::reminder::send_verification(&phone_number)
        .await
        .map_err(|e| {
            errors::ServerError::WidgetTemplateError(format!("otp-send-verification-template: {e}"))
        })?;

    cookie
        .set::<String>(consts::OTP_PHONE_COOKIE_NAME, phone_number.to_string())
        .map_err(|e| {
            errors::ServerError::WidgetTemplateError(format!("otp-set-cookie-session: {e}"))
        })?;

    let content = templates::WEB_TEMPLATES
        .render("widgets/otp.html", &context)
        .unwrap_or("Intente mas tarde".to_string());

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(content))
}

#[web::post("/verify-otp")]
async fn verify_reminder_phone(
    _: CheckUserCanAccessService,
    mut user_session: session::WebAppSession,
    form: web::types::Form<forms::user::ReminderPhoneOtp>,
    app_state: web::types::State<AppState>,
    cookie: Session,
    identity: Identity,
    _: CsrfToken,
) -> Result<impl web::Responder, web::Error> {
    let mut context = tera::Context::from_value(json!({
        "otp_step": "OTP_FAILURE",
    }))
    .unwrap_or_default();

    if api::reminder::validate_otp(&form.otp_value).await {
        if let Ok(Some(phone_number)) = cookie.get::<String>(consts::OTP_PHONE_COOKIE_NAME) {
            if api::reminder::add_verified_phone_to_user(
                user_session.user.id,
                &phone_number,
                &app_state.repo,
            )
            .await
            .is_ok()
            {
                user_session.user.phone_reminder = Some(phone_number.to_string());
                identity.remember(
                    serde_json::to_string(&user_session).unwrap(), //unwrap cause its safe, it comes internally
                );

                cookie.remove(consts::OTP_PHONE_COOKIE_NAME);

                context.insert("otp_step", "OTP_SUCCESS");
                context.insert("phone_reminder", &phone_number);
            }
        }
    };

    let content = templates::WEB_TEMPLATES
        .render("widgets/otp.html", &context)
        .unwrap_or("Intente mas tarde".to_string());

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(content))
}

#[web::delete("/verified-phone")]
async fn remove_verified_phone(
    _: CheckUserCanAccessService,
    mut user_session: session::WebAppSession,
    app_state: web::types::State<AppState>,
    identity: Identity,
    _: CsrfToken,
) -> Result<impl web::Responder, web::Error> {
    let mut context = tera::Context::from_value(json!({
        "otp_step": "OTP_SUCCESS",
        "phone_reminder": user_session.user.phone_reminder.unwrap(),
    }))
    .unwrap_or_default();

    if api::reminder::remove_verified_phone_to_user(user_session.user.id, &app_state.repo)
        .await
        .is_ok()
    {
        user_session.user.phone_reminder = None;
        identity.remember(
            serde_json::to_string(&user_session).unwrap(), //unwrap cause its safe, it comes internally
        );

        context.insert("otp_step", "OTP_START");
    }

    let content = templates::WEB_TEMPLATES
        .render("widgets/otp.html", &context)
        .unwrap_or("Intente mas tarde".to_string());

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(content))
}

/// Handles the request to delete a reminder
#[web::delete("/{reminder_id}")]
async fn delete_reminder(
    _: IsUserLoggedAndCanEdit,
    session::WebAppSession { user, .. }: session::WebAppSession,
    params: web::types::Path<(i64,)>,
    app_state: web::types::State<AppState>,
    _: CsrfToken,
) -> Result<impl web::Responder, web::Error> {
    let reminder_id = params.0;

    api::reminder::delete_reminder(
        reminder_id,
        user.id,
        &app_state.repo,
        &app_state.notification_service,
    )
    .await
    .map_err(|e| {
        errors::ServerError::ExternalServiceError(format!(
            "function delete_reminder raised an error: {e}"
        ))
    })?;

    Ok(web::HttpResponse::Ok()
        .set_header("HX-Trigger", "reminderRecordUpdated")
        .body(""))
}

/// Handles the request to create a reminder
#[web::post("")]
async fn create_reminder(
    _: IsUserLoggedAndCanEdit,
    session::WebAppSession { user, .. }: session::WebAppSession,
    r: ntex::web::HttpRequest,
    form: web::types::Form<forms::user::UserReminderForm>,
    app_state: web::types::State<AppState>,
    _: CsrfToken,
) -> Result<impl web::Responder, web::Error> {
    let user_timezone: Tz =
        utils::extract_usertimezone(r.headers()).unwrap_or(Tz::America__Mexico_City);

    if user.phone_reminder.is_none() {
        return Ok(web::HttpResponse::BadRequest()
            .content_type("text/html; charset=utf-8")
            .body(""));
    }

    if let Some(user_dt) = form.when.and_local_timezone(user_timezone).single() {
        api::reminder::schedule_reminder(
            api::reminder::ScheduleReminderInfo {
                user_id: user.id,
                phone_number: user.phone_reminder.unwrap(),
                when: user_dt,
                body: form.body.to_string(),
            },
            &app_state.repo,
            &app_state.notification_service,
        )
        .await
        .map_err(|e| {
            errors::ServerError::InternalServerError(format!(
                "function schedule_reminder raised an error: {e}"
            ))
        })?;
    }

    Ok(web::HttpResponse::Created()
        .content_type("text/html; charset=utf-8")
        .set_header("HX-Trigger", "reminderRecordUpdated")
        .body(""))
}
