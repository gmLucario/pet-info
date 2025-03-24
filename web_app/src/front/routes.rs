use super::{blog, checkout, pet, pet_health, pet_note, profile, reminder};
use ntex::web;

pub fn pet(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/pet").service((
        pet::get_pet_view,
        pet::user_pets_list,
        pet::get_pet_info_view,
        pet::render_pet_details_form,
        pet::create_pet_request,
        pet::get_profile_qr_code,
        pet::get_pet_public_pic,
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

pub fn profile(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/profile").service((
        profile::get_profile_view,
        profile::add_new_owner_contact,
        profile::get_owner_contacts,
        profile::delete_owner_contact,
        profile::delete_user_data,
    )));
}

pub fn checkout(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/checkout").service((checkout::get_checkout_view, checkout::process_payment)),
    );
}

pub fn blog(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/blog").service((blog::get_blog_entry,)));
}
