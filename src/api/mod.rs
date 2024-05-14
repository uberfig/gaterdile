use rocket::fairing::AdHoc;

pub mod messages;
pub mod sessions;

pub fn stage_api() -> AdHoc {
    AdHoc::on_ignite("api", |rocket| async {
        rocket
            .mount("/api", sessions::routes())
            .mount("/api", messages::routes())
    })
}
