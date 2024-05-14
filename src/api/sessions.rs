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
        AuthErr::Success(x) => {
            jar.add_private(("user_id", format!("{x}")))
        },
        AuthErr::InvalidUsername => {},
        AuthErr::InvalidPassword => {},
    };

    Ok(id.into())

    // match id {
    //     Err(rusqlite::Error::QueryReturnedNoRows) => Ok(AuthResult::Failed.into()),
    //     Ok(id) => {
    //         jar.add_private(Cookie::build(("user_id", format!("{id}"))));
    //         jar.add_private(("session_id", format!("{}", sessions.next_session_id().0)));
    //         Ok(AuthResult::Authorized.into())
    //     }
    //     Err(err) => Err(err.into()),
    // }
}

pub fn routes() -> Vec<Route> {
    routes![
        login,
    ]
}
