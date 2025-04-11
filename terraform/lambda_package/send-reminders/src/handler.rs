use lambda_runtime::{tracing, Error, LambdaEvent};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::config;

#[derive(Deserialize, Debug)]
pub struct IncomingMessage {
    phone: String,
    body: String,
}

#[derive(Serialize)]
pub struct OutgoingMessage {
    req_id: String,
    msg: String,
}

async fn send_msg(phone_number: &str, body: &str) -> Result<(), Error> {
    let response = reqwest::Client::new()
        .post(config::APP_CONFIG.whatsapp_send_msg_endpoint())
        .header("accept", "application/json")
        .header("content-type", "application/json")
        .bearer_auth(config::APP_CONFIG.whatsapp_business_auth.to_string())
        .json(&json!({
        "messaging_product": "whatsapp",
        "recipient_type": "individual",
        "to": phone_number,
        "type": "template",
        "template": {
            "name": "custom_reminder_es_mex",
            "language": {"code": "es_mx"},
            "components": [{"type": "body", "parameters": [{"type": "text","text": body}]}]
        }}))
        .send()
        .await?;

    if response.status().is_success() {
        return Ok(());
    }

    let response = response
        .json::<serde_json::Value>()
        .await
        .unwrap_or_default();

    Err(Box::new(simple_error::SimpleError::new(format!(
        "remainder did not send: {response}"
    ))))
}

#[tracing::instrument()]
pub async fn function_handler(
    event: LambdaEvent<IncomingMessage>,
) -> Result<OutgoingMessage, Error> {
    let payload = event.payload;

    send_msg(&payload.phone, &payload.body).await?;

    Ok(OutgoingMessage {
        req_id: event.context.request_id,
        msg: "reminder was sent".into(),
    })
}
