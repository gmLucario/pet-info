use crate::models;

#[derive(serde::Deserialize, Debug, Default)]
pub struct CardFormData {
    pub installments: u32,
    pub issuer_id: String,
    pub payer: models::mp_paym::PayerInfo,
    pub payment_method_id: String,
    pub token: String,
}
