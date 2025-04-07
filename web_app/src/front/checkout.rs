use crate::{
    api, config, consts,
    front::{AppState, errors, middleware, templates},
    models,
};
use ntex::web;
use ntex_identity::Identity;
use serde_json::json;

#[web::get("")]
async fn get_checkout_view(
    logged_user: models::user_app::User,
) -> Result<impl web::Responder, web::Error> {
    let context = tera::Context::from_value(json!({
        "service_price": format!("{:.2}", consts::SERVICE_PRICE),
        "email": &logged_user.email,
        "mercado_pago_public_key": &config::APP_CONFIG.mercado_pago_public_key,
        "back_error_url": format!("{}://{}/profile", config::APP_CONFIG.wep_server_protocol(), config::APP_CONFIG.url_host())
    }))
    .unwrap_or_default();

    let content = templates::WEB_TEMPLATES
        .render("checkout.html", &context)
        .map_err(|e| {
            errors::ServerError::TemplateError(format!(
                "at /checkout endpoint the template couldnt be rendered: {}",
                e
            ))
        })?;

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(content))
}

#[derive(serde::Serialize)]
struct PaymResponse {
    id: usize,
}

#[web::post("/process_payment")]
async fn process_payment(
    logged_user: models::user_app::User,
    app_state: web::types::State<AppState>,
    request_body: web::types::Json<models::mp_paym::PaymentInfo>,
    identity: Identity,
    _: middleware::csrf_token::CsrfToken,
) -> Result<impl web::Responder, web::Error> {
    let payment_info = request_body.0;

    let (mp_paym_id, is_subscribed) = api::payment::create_subscription(
        &app_state.repo,
        api::payment::PaymentSubsRequest {
            user_id: logged_user.id,
            transaction_amount: consts::SERVICE_PRICE.to_string(),
            mp_paym_info: payment_info,
        },
    )
    .await
    .map_err(|e| {
        errors::ServerError::ExternalServiceError(format!(
            "at create_subscription::{}: {}",
            logged_user.email, e
        ))
    })?;

    identity.remember(
        serde_json::to_string(&models::user_app::User {
            is_subscribed,
            ..logged_user
        })
        .unwrap(),
    );

    Ok(web::HttpResponse::Created().json(&PaymResponse { id: mp_paym_id }))
}
