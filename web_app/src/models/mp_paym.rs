#[derive(serde::Deserialize, serde::Serialize, Debug, Default, Clone)]
pub struct PayerInfo {
    pub email: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
pub struct PaymentInfo {
    pub installments: u32,
    pub issuer_id: String,
    pub payer: PayerInfo,
    pub payment_method_id: String,
    pub token: String,
    #[serde(default = "default_sub_paym_desc")]
    pub description: String,
    pub transaction_amount: f32,
}

pub fn default_sub_paym_desc() -> String {
    "pet info web app sub".into()
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Default)]
pub struct PaymentResponse {
    pub id: usize,
    pub status: super::payment::PaymentStatus,
}
