use rocket::http::{Cookie, CookieJar, Status};
use rocket::Route;
use rocket::{get, post, routes, State};

use crate::database::db::DbConn;
use crate::database::users::auth;
use crate::transmission::{AuthErr, UserAuth};
use rocket::serde::json::Json;
use rocket_db_pools::Connection;

#[post("/login", data = "<user_auth>")]
async fn login(
    mut conn: Connection<DbConn>,
    jar: &CookieJar<'_>,
    // sessions: &State<SessionManager>,
    user_auth: Json<UserAuth>,
) -> Result<Json<AuthErr>, ()> {
    jar.remove_private("user_id");

    let id = auth(user_auth.into_inner(), &mut conn).await;

    match id {
        AuthErr::Success(x) => jar.add_private(("user_id", format!("{x}"))),
        AuthErr::InvalidUsername => {}
        AuthErr::InvalidPassword => {}
    };

    Ok(id.into())
}

#[post("/logout")]
async fn logout(jar: &CookieJar<'_>) {
    jar.remove_private("user_id");
}

pub fn routes() -> Vec<Route> {
    routes![login, logout,]
}
