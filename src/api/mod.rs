use rocket::fairing::AdHoc;

pub mod sessions;

pub fn stage_api() -> AdHoc {
    AdHoc::on_ignite("api", |rocket| async {
        rocket.mount("/api", sessions::routes())
    })
}
