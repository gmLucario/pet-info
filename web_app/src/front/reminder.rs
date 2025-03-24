use crate::{
    api, consts,
    front::{
        errors, forms,
        middleware::{
            csrf_token::CsrfToken,
            logged_user::{CheckUserCanAccessService, IsUserLoggedAndCanEdit},
        },
        templates, AppState,
    },
    models,
};
use chrono_tz::Tz;
use ntex::web::{self};
use ntex_identity::Identity;
use ntex_session::Session;
use serde_json::json;

#[web::get("")]
async fn get_reminder_view(
    _: IsUserLoggedAndCanEdit,
    logged_user: models::user_app::User,
    app_state: web::types::State<AppState>,
) -> Result<impl web::Responder, web::Error> {
    let context = tera::Context::from_value(json!({
        "reminders": api::reminder::get_scheduled_reminders(logged_user.id, &app_state.repo).await.unwrap_or_default(),
    })).unwrap_or_default();

    let content = templates::WEB_TEMPLATES
        .render("reminders.html", &context)
        .map_err(|e| {
            errors::ServerError::TemplateError(format!(
                "at /reminder endpoint the template couldnt be rendered: {}",
                e
            ))
        })?;

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(content))
}

#[web::get("/tbody")]
async fn get_reminder_records(
    _: IsUserLoggedAndCanEdit,
    logged_user: models::user_app::User,
    app_state: web::types::State<AppState>,
) -> Result<impl web::Responder, web::Error> {
    let context = tera::Context::from_value(json!({
        "reminders": api::reminder::get_scheduled_reminders(logged_user.id, &app_state.repo).await.unwrap_or_default(),
    })).unwrap_or_default();

    let content = templates::WEB_TEMPLATES
        .render("widgets/tbody_reminder.html", &context)
        .map_err(|e| {
            errors::ServerError::WidgetTemplateError(format!(
                "at /reminder/tbody endpoint the template couldnt be rendered: {}",
                e
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

    let form = form.0;
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
    mut logged_user: models::user_app::User,
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
                logged_user.id,
                &phone_number,
                &app_state.repo,
            )
            .await
            .is_ok()
            {
                logged_user.phone_reminder = Some(phone_number.to_string());
                let user_string = serde_json::to_string(&logged_user).unwrap(); //unwrap cause its safe, it comes internally
                identity.remember(user_string);

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
    mut logged_user: models::user_app::User,
    app_state: web::types::State<AppState>,
    identity: Identity,
    _: CsrfToken,
) -> Result<impl web::Responder, web::Error> {
    let mut context = tera::Context::from_value(json!({
        "otp_step": "OTP_SUCCESS",
        "phone_reminder": logged_user.phone_reminder.unwrap(),
    }))
    .unwrap_or_default();

    if api::reminder::remove_verified_phone_to_user(logged_user.id, &app_state.repo)
        .await
        .is_ok()
    {
        logged_user.phone_reminder = None;
        let user_string = serde_json::to_string(&logged_user).unwrap(); //unwrap cause its safe, it comes internally
        identity.remember(user_string);

        context.insert("otp_step", "OTP_START");
    }

    let content = templates::WEB_TEMPLATES
        .render("widgets/otp.html", &context)
        .unwrap_or("Intente mas tarde".to_string());

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(content))
}

#[web::delete("/{reminder_id}")]
async fn delete_reminder(
    _: IsUserLoggedAndCanEdit,
    logged_user: models::user_app::User,
    params: web::types::Path<(i64,)>,
    app_state: web::types::State<AppState>,
    _: CsrfToken,
) -> Result<impl web::Responder, web::Error> {
    let reminder_id = params.0;

    api::reminder::delete_reminder(
        reminder_id,
        logged_user.id,
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

#[web::post("")]
async fn create_reminder(
    _: IsUserLoggedAndCanEdit,
    logged_user: models::user_app::User,
    r: ntex::web::HttpRequest,
    form: web::types::Form<forms::user::UserReminderForm>,
    app_state: web::types::State<AppState>,
    _: CsrfToken,
) -> Result<impl web::Responder, web::Error> {
    let request_headers = r.headers();

    let user_timezone = request_headers
        .get("timezone")
        .map(|v| v.to_str().unwrap_or("America/Mexico_City"))
        .unwrap_or("America/Mexico_City");
    let user_timezone: Tz = user_timezone.parse().unwrap_or(Tz::America__Mexico_City);

    if let Some(user_dt) =
        chrono::NaiveDateTime::parse_from_str(&form.when, consts::DATETIME_LOCAL_INPUT_FORMAT)
            .map_err(|e| errors::UserError::FormInputValueError(format!("invalid timezone: {e}")))?
            .and_local_timezone(user_timezone)
            .earliest()
    {
        api::reminder::schedule_reminder(
            api::reminder::ScheduleReminderInfo {
                user_id: logged_user.id,
                phone_number: logged_user.phone_reminder.unwrap(),
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

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .set_header("HX-Trigger", "reminderRecordUpdated")
        .body("content"))
}
