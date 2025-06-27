//! Frontend route configuration module.
//!
//! This module organizes and configures all the web routes for the pet info application.
//! Routes are grouped by functionality into logical scopes for better organization
//! and maintainability.

use super::{blog, checkout, pet, pet_health, pet_note, pet_public, profile, reminder};
use ntex::web;

/// Configures public pet profile routes.
///
/// This function sets up routes for viewing pet information without authentication.
/// These routes are typically used for public pet profiles that can be accessed
/// via QR codes or direct links.
///
/// # Routes
/// - `GET /info/{pet_external_id}` - View public pet information
pub fn pet_public_profile(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/info").service((pet_public::get_pet_info_view,)));
}

/// Configures pet management routes.
///
/// This function sets up all routes related to pet CRUD operations, including
/// pet details, health records, notes, and file downloads. All routes require
/// user authentication and proper authorization.
///
/// # Main Routes
/// - `GET /pet` - Pet management dashboard
/// - `GET /pet/list` - List user's pets
/// - `GET /pet/details/{pet_id}` - Pet details form
/// - `POST /pet/create` - Create new pet
/// - `PUT /pet/edit/{pet_id}` - Update pet details
/// - `DELETE /pet/delete/{pet_id}` - Delete pet
/// - `GET /pet/qr_code/{pet_external_id}` - Generate QR code
/// - `GET /pet/pdf_report/{pet_id}` - Generate PDF report
/// - `GET /pet/public_pic/{pet_external_id}` - Get pet picture
/// - `GET /pet/pass/{pet_external_id}` - Download Apple Wallet pass
///
/// # Health Sub-routes (/pet/health)
/// - `GET /pet/health/{pet_external_id}/{health_type}` - Health records view
/// - `POST /pet/health/add` - Add health record
/// - `DELETE /pet/health/delete` - Delete health record
///
/// # Notes Sub-routes (/pet/note)
/// - `GET /pet/note/{pet_id}` - Pet notes view
/// - `POST /pet/note/new` - Create new note
/// - `GET /pet/note/list/{pet_id}` - Get pet notes
/// - `DELETE /pet/note/delete` - Delete note
pub fn pet(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/pet").service((
        pet::get_pet_view,
        pet::user_pets_list,
        pet::render_pet_details_form,
        pet::create_pet_request,
        pet::get_profile_qr_code,
        pet::get_pdf_report,
        pet::get_pet_public_pic,
        pet::download_pet_pass,
        pet::delete_pet,
        pet::get_pet_details_form,
        pet::edit_pet_details,
        web::scope("/health").service((
            pet_health::get_pet_health_view,
            pet_health::pet_health_records,
            pet_health::add_health_record,
            pet_health::delete_health_record,
        )),
        web::scope("/note").service((
            pet_note::get_pet_notes_view,
            pet_note::new_pet_note,
            pet_note::get_pet_notes,
            pet_note::delete_pet_note,
        )),
    )));
}

/// Configures reminder management routes.
///
/// This function sets up routes for managing pet care reminders, including
/// phone number verification for SMS notifications. All routes require
/// user authentication.
///
/// # Routes
/// - `GET /reminder` - Reminders management view
/// - `GET /reminder/list` - Get user's reminders
/// - `POST /reminder/create` - Create new reminder
/// - `DELETE /reminder/delete/{reminder_id}` - Delete reminder
/// - `POST /reminder/phone/start-verification` - Start phone verification
/// - `POST /reminder/phone/send-code` - Send verification code
/// - `POST /reminder/phone/verify` - Verify phone number
/// - `DELETE /reminder/phone/remove` - Remove verified phone
pub fn reminders(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/reminder").service((
        reminder::get_reminder_view,
        reminder::get_reminder_records,
        reminder::send_verification_code_to_reminder_phone,
        reminder::verify_reminder_phone,
        reminder::start_verification_code_to_reminder_phone,
        reminder::remove_verified_phone,
        reminder::create_reminder,
        reminder::delete_reminder,
    )));
}

/// Configures user profile management routes.
///
/// This function sets up routes for managing user account settings, contact
/// information, and account operations. All routes require user authentication.
///
/// # Routes
/// - `GET /profile` - User profile management view
/// - `POST /profile/contact/add` - Add new owner contact
/// - `GET /profile/contact/list` - Get owner contacts
/// - `DELETE /profile/contact/delete/{contact_id}` - Delete owner contact
/// - `DELETE /profile/delete-data` - Delete all user data
/// - `POST /profile/logout` - Close user session
pub fn user_profile(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/profile").service((
        profile::get_profile_view,
        profile::add_new_owner_contact,
        profile::get_owner_contacts,
        profile::delete_owner_contact,
        profile::delete_user_data,
        profile::close_session,
    )));
}

/// Configures payment and subscription checkout routes.
///
/// This function sets up routes for handling subscription payments and
/// checkout processes. Routes require user authentication.
///
/// # Routes
/// - `GET /checkout` - Checkout page view
/// - `POST /checkout/process` - Process payment
pub fn checkout(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/checkout").service((checkout::get_checkout_view, checkout::process_payment)),
    );
}

/// Configures blog and content routes.
///
/// This function sets up routes for serving blog content and informational
/// pages. These routes are typically public and don't require authentication.
///
/// # Routes
/// - `GET /blog/{entry_id}` - View blog entry
pub fn blog(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/blog").service((blog::get_blog_entry,)));
}
