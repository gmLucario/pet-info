use rust_decimal::Decimal;

#[derive(serde::Deserialize, serde::Serialize, Debug, Default, Clone)]
pub struct PayerInfo {
    pub email: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Default, Clone)]
pub struct MercadoPagoPaymentRequest {
    pub installments: u32,
    pub issuer_id: String,
    pub payer: PayerInfo,
    pub payment_method_id: String,
    pub token: String,
    pub description: String,
    #[serde(serialize_with = "rust_decimal::serde::float::serialize")]
    pub transaction_amount: Decimal,
}

pub fn default_sub_paym_desc() -> String {
    "pet info web app pet subs".into()
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
pub struct PaymentResponse {
    pub id: usize,
    pub status: super::payment::PaymentStatus,
}
