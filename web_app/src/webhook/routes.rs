use ntex::web;

/// Configures webhook routes for external integrations.
///
/// This function sets up routes for receiving webhook events from external
/// services like WhatsApp Business API. These routes are public endpoints
/// that don't require authentication.
///
/// # Routes
/// - `GET /webhook/whatsapp` - WhatsApp webhook verification
/// - `POST /webhook/whatsapp` - WhatsApp webhook receiver
pub fn whatsapp(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/webhook/whatsapp")
            .service((super::whatsapp::verify, super::whatsapp::receive)),
    );
}
