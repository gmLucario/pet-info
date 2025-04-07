use anyhow::bail;
use chrono::Utc;
use log::error;
use serde_json::from_str;
use uuid::Uuid;

use crate::{config, consts, models, repo, utils};

pub struct PaymentSubsRequest {
    pub user_id: i64,
    pub transaction_amount: String,
    pub mp_paym_info: models::mp_paym::PaymentInfo,
}

impl From<PaymentSubsRequest> for models::payment::Payment {
    fn from(val: PaymentSubsRequest) -> Self {
        let now = Utc::now();
        Self {
            user_id: val.user_id,
            mp_paym_id: 0,
            payment_idempotency_h: Uuid::new_v4().into(),
            transaction_amount: consts::SERVICE_PRICE.to_string(),
            installments: val.mp_paym_info.installments,
            payment_method_id: val.mp_paym_info.payment_method_id.to_string(),
            issuer_id: val.mp_paym_info.issuer_id.to_string(),
            status: models::payment::PaymentStatus::default(),
            created_at: now,
            updated_at: now,
        }
    }
}

pub async fn create_subscription(
    repo: &repo::ImplAppRepo,
    payment_request: PaymentSubsRequest,
) -> anyhow::Result<(usize, bool)> {
    if let Some(subs_payment) = repo
        .get_user_payments(payment_request.user_id)
        .await?
        .first()
    {
        if subs_payment.is_approved() {
            return Ok((subs_payment.mp_paym_id, true));
        }
    }

    let token = payment_request.mp_paym_info.token.to_string();
    let payer = payment_request.mp_paym_info.payer.clone();

    let mut subs_payment: models::payment::Payment = payment_request.into();

    let response = utils::REQUEST_CLIENT
        .post("https://api.mercadopago.com/v1/payments")
        .header("accept", "application/json")
        .header("content-type", "application/json")
        .header("X-Idempotency-Key", &subs_payment.payment_idempotency_h)
        .bearer_auth(&config::APP_CONFIG.mercado_token)
        .json(&models::mp_paym::PaymentInfo {
            installments: subs_payment.installments,
            issuer_id: subs_payment.issuer_id.clone(),
            payer,
            token,
            payment_method_id: subs_payment.payment_method_id.clone(),
            // external_reference: subs_payment.create_external_reference(),
            transaction_amount: from_str::<f32>(&consts::SERVICE_PRICE.to_string())
                .unwrap_or(500f32),
            description: models::mp_paym::default_sub_paym_desc(),
        })
        .send()
        .await
        .unwrap();

    if !response.status().is_success() {
        error!("{:#?}", response.json::<serde_json::Value>().await);
        bail!("mercado pago api is returning an error");
    }

    let body_response = response.json::<models::mp_paym::PaymentResponse>().await?;

    subs_payment.status = body_response.status;
    subs_payment.mp_paym_id = body_response.id;

    repo.set_user_as_subscribed(subs_payment.user_id).await?;
    repo.save_subs_payment(&subs_payment).await?;

    Ok((subs_payment.mp_paym_id, subs_payment.is_approved()))
}
