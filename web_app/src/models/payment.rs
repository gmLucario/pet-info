use chrono::{DateTime, Utc};
use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Serialize, PartialEq, Display)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    // Firts time user try to pay
    #[default]
    #[display("initiated")]
    Initiated,
    // El pago ha sido aprobado y acreditado con éxito
    #[display("approved")]
    Approved,
    // El pago ha sido autorizado pero aún no se ha capturado
    #[display("authorized")]
    Authorized,
    // El pago está en proceso de revisión
    #[display("in_process")]
    InProcess,
    // El pago fue rechazado (el usuario puede intentar pagar nuevamente)
    #[display("rejected")]
    Rejected,
    // El pago fue cancelado por alguna de las partes o caducó
    #[display("cancelled")]
    Cancelled,
    // El pago fue reembolsado al usuario.
    #[display("refunded")]
    Refunded,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Payment {
    pub user_id: i64,
    pub mp_paym_id: usize,
    pub payment_idempotency_h: String,
    pub transaction_amount: String,
    pub installments: u32,
    pub payment_method_id: String,
    pub issuer_id: String,
    pub status: PaymentStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Payment {
    pub fn create_external_reference(&self) -> String {
        let user_id = self.user_id;
        format!("PetInfoSubsPayment{user_id}")
    }

    pub fn is_approved(&self) -> bool {
        self.status.eq(&PaymentStatus::Approved)
    }
}
