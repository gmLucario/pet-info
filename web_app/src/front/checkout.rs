//! Handlers related to the PetInfo id tag payment

use crate::{
    api, config, consts,
    front::{AppState, errors, forms, middleware, session, templates, utils},
    models,
};
use anyhow::Context;
use ntex::web;
use ntex_identity::Identity;
use serde_json::json;

/// Endpoint to render the checkout to pay the PetInfo id tag.
/// If the user payed without finish the id tag register, the user
/// will be redirect to the add form
#[web::get("")]
async fn get_checkout_view(
    user_session: session::WebAppSession,
) -> Result<impl web::Responder, web::Error> {
    if user_session.has_pet_balance() {
        return utils::redirect_to("pet/new");
    }

    let app_config = config::APP_CONFIG
        .get()
        .context("failed to get app config")
        .map_err(|e| web::error::ErrorInternalServerError(e))?;

    let context = tera::Context::from_value(json!({
        "service_price": format!("{:.2}", consts::ADD_PET_PRICE),
        "email": &user_session.user.email,
        "mercado_pago_public_key": &app_config.mercado_pago_public_key,
        "back_url": format!("{}/pet", app_config.base_url()),
    }))
    .unwrap_or_default();

    let content = templates::WEB_TEMPLATES
        .render("checkout.html", &context)
        .map_err(|e| {
            errors::ServerError::TemplateError(format!(
                "at /checkout endpoint the template couldnt be rendered: {e}"
            ))
        })?;

    Ok(web::HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(content))
}

/// Represents the process_payment response endpoint [post]
#[derive(serde::Serialize)]
struct PaymResponse {
    id: usize,
}

/// Endpoint to handle the payment request made by the user to adquire a
/// PetInfo id tag.
#[web::post("/process_payment")]
async fn process_payment(
    _: middleware::csrf_token::CsrfToken,
    session::WebAppSession {
        mut user,
        add_pet_balance,
    }: session::WebAppSession,
    app_state: web::types::State<AppState>,
    request_body: web::types::Json<forms::payment::CardFormData>,
    identity: Identity,
) -> Result<impl web::Responder, web::Error> {
    let _span = logfire::span!("process_payment").entered();

    let (mp_paym_id, is_subscribed) = api::payment::create_subscription(
        &app_state.repo,
        api::payment::PaymentSubsRequest {
            user_id: user.id,
            transaction_amount: consts::ADD_PET_PRICE,
            mp_paym_info: models::mp_paym::MercadoPagoPaymentRequest {
                installments: request_body.installments,
                issuer_id: request_body.issuer_id.to_string(),
                payer: request_body.payer.clone(),
                payment_method_id: request_body.payment_method_id.to_string(),
                token: request_body.token.to_string(),
                description: models::mp_paym::default_sub_paym_desc(),
                transaction_amount: consts::ADD_PET_PRICE,
            },
        },
        add_pet_balance, // balance were 0 check
    )
    .await
    .map_err(|e| {
        errors::ServerError::ExternalServiceError(format!(
            "at create_subscription::{}: {}",
            user.email, e
        ))
    })?;

    user.is_subscribed = user.is_subscribed || is_subscribed;
    let add_pet_balance = add_pet_balance + u32::from(is_subscribed);

    identity.remember(
        serde_json::to_string(&session::WebAppSession {
            user,
            add_pet_balance,
        })
        .map_err(|e| {
            errors::ServerError::InternalServerError(format!(
                "at process_payment::identity::remember: {e}"
            ))
        })?,
    );

    Ok(web::HttpResponse::Created().json(&PaymResponse { id: mp_paym_id }))
}
