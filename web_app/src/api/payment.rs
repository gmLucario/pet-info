use anyhow::bail;
use chrono::Utc;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::{config, metric, models, repo, utils};

#[derive(Debug)]
pub struct PaymentSubsRequest {
    pub user_id: i64,
    pub transaction_amount: Decimal,
    pub mp_paym_info: models::mp_paym::MercadoPagoPaymentRequest,
}

/// If user_approved_payments > user pets. User hasnt completed the last pet form flow
pub async fn user_has_orphan_payment(
    repo: &repo::ImplAppRepo,
    user_id: i64,
) -> anyhow::Result<bool> {
    let user_approved_payments = repo
        .get_user_payments(user_id, Some(models::payment::PaymentStatus::Approved))
        .await?
        .len();
    let pets = repo.get_all_pets_user_id(user_id).await?.len();

    Ok(user_approved_payments > pets)
}

pub async fn create_subscription(
    repo: &repo::ImplAppRepo,
    payment_request: PaymentSubsRequest,
    pet_balance: u32,
) -> anyhow::Result<(usize, bool)> {
    let _span = logfire::span!("create_subscription").entered();

    let payment_idempotency_h = Uuid::new_v4().to_string();
    let response = utils::REQUEST_CLIENT
        .post("https://api.mercadopago.com/v1/payments")
        .header("accept", "application/json")
        .header("content-type", "application/json")
        .header("X-Idempotency-Key", &payment_idempotency_h)
        .bearer_auth(&config::APP_CONFIG.mercado_token)
        .json(&payment_request.mp_paym_info)
        .send()
        .await
        .unwrap();

    if !response.status().is_success() {
        logfire::error!(
            "mp_response={mp_response}",
            mp_response = response
                .json::<serde_json::Value>()
                .await
                .unwrap_or_default()
                .to_string()
        );

        bail!("mercado pago api is returning an error");
    }

    let body_response = response.json::<models::mp_paym::PaymentResponse>().await?;
    let now = Utc::now();

    let subs_payment = models::payment::Payment {
        mp_paym_id: body_response.id,
        user_id: payment_request.user_id,
        payment_idempotency_h,
        transaction_amount: payment_request.mp_paym_info.transaction_amount.to_string(),
        installments: payment_request.mp_paym_info.installments,
        payment_method_id: payment_request.mp_paym_info.payment_method_id,
        issuer_id: payment_request.mp_paym_info.issuer_id,
        status: body_response.status,
        created_at: now,
        updated_at: now,
    };

    let is_subscribed = subs_payment.is_approved();

    repo.set_user_as_subscribed(subs_payment.user_id).await?;
    repo.save_subs_payment(&subs_payment).await?;
    repo.set_pet_balance(subs_payment.user_id, pet_balance + u32::from(is_subscribed))
        .await?;

    metric::incr_payment_status_statds(&subs_payment.status.to_string().to_lowercase());
    Ok((subs_payment.mp_paym_id, is_subscribed))
}
