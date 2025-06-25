//! # Payment API Module
//!
//! This module handles all payment-related operations including subscription
//! management, payment processing through MercadoPago, and user balance tracking.

use anyhow::bail;
use chrono::Utc;
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::{config, metric, models, repo, utils};

/// Payment subscription request structure.
///
/// Contains all necessary information to process a subscription payment
/// through the MercadoPago payment gateway.
#[derive(Debug)]
pub struct PaymentSubsRequest {
    /// ID of the user making the payment
    pub user_id: i64,
    /// Amount to be charged for the subscription
    pub transaction_amount: Decimal,
    /// MercadoPago-specific payment information
    pub mp_paym_info: models::mp_paym::MercadoPagoPaymentRequest,
}

/// Checks if a user has orphaned payments.
///
/// Determines if a user has approved payments that don't correspond to completed
/// pet registrations. This happens when a user pays for a pet but doesn't complete
/// the pet creation form flow.
///
/// # Arguments
/// * `repo` - Repository instance for database operations
/// * `user_id` - ID of the user to check
///
/// # Returns
/// * `anyhow::Result<bool>` - True if user has more approved payments than pets
///
/// # Logic
/// If `approved_payments > pet_count`, the user has orphaned payments that need
/// to be resolved before allowing new pet creation.
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

/// Calls the MercadoPago API to process a payment.
///
/// Makes an HTTP request to the MercadoPago payments API with the provided
/// payment information and idempotency key. Handles error responses and
/// parses the successful response.
///
/// # Arguments
/// * `payment_info` - MercadoPago payment request data
/// * `idempotency_key` - Unique key to ensure payment idempotency
///
/// # Returns
/// * `anyhow::Result<models::mp_paym::PaymentResponse>` - Parsed payment response from API
///
/// # Errors
/// Returns an error if:
/// - HTTP request fails
/// - MercadoPago API returns error status
/// - Response parsing fails
/// - Network connectivity issues
///
/// # Process
/// 1. Construct HTTP request with proper headers
/// 2. Send payment data to MercadoPago API
/// 3. Validate response status
/// 4. Parse and return payment response
async fn call_mercado_pago_api(
    payment_info: &models::mp_paym::MercadoPagoPaymentRequest,
    idempotency_key: &str,
) -> anyhow::Result<models::mp_paym::PaymentResponse> {
    let response = utils::REQUEST_CLIENT
        .post("https://api.mercadopago.com/v1/payments")
        .header("accept", "application/json")
        .header("content-type", "application/json")
        .header("X-Idempotency-Key", idempotency_key)
        .bearer_auth(&config::APP_CONFIG.mercado_token)
        .json(payment_info)
        .send()
        .await?;

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

    response
        .json::<models::mp_paym::PaymentResponse>()
        .await
        .map_err(Into::into)
}

/// Creates a subscription payment through MercadoPago.
///
/// Processes a subscription payment request by calling the MercadoPago API,
/// storing the payment information, and updating the user's subscription status
/// and pet balance accordingly.
///
/// # Arguments
/// * `repo` - Repository instance for database operations
/// * `payment_request` - Payment details and MercadoPago information
/// * `pet_balance` - Current pet balance for the user
///
/// # Returns
/// * `anyhow::Result<(usize, bool)>` - Tuple of (payment_id, is_approved)
///
/// # Process Flow
/// 1. Generate idempotency key for payment safety
/// 2. Call MercadoPago API with payment details
/// 3. Parse response and create payment record
/// 4. Update user subscription status
/// 5. Increment pet balance if payment approved
/// 6. Record metrics for monitoring
///
/// # Errors
/// Returns an error if:
/// - MercadoPago API call fails
/// - Database operations fail
/// - Response parsing fails
pub async fn create_subscription(
    repo: &repo::ImplAppRepo,
    payment_request: PaymentSubsRequest,
    pet_balance: u32,
) -> anyhow::Result<(usize, bool)> {
    let _span = logfire::span!("create_subscription").entered();

    let payment_idempotency_h = Uuid::new_v4().to_string();
    let body_response =
        call_mercado_pago_api(&payment_request.mp_paym_info, &payment_idempotency_h).await?;
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
