use rocket::http::CookieJar;
use rocket::Route;
use rocket::{post, routes};
use serde::{Deserialize, Serialize};

use crate::database::db::DbConn;
use crate::database::db_types::Message;
use crate::transmission::NewTransmissionMessage;
use rocket::serde::json::Json;
use rocket_db_pools::Connection;

#[derive(Debug, Serialize, Deserialize)]
struct MessageId(i64);

///TODO check that the user has posting privilage in the given room
#[post("/send_message", data = "<message>")]
async fn send_message(
    mut conn: Connection<DbConn>,
    jar: &CookieJar<'_>,
    message: Json<NewTransmissionMessage>,
) -> Result<Json<MessageId>, ()> {
    if message.text.trim().is_empty() {
        return Err(());
    }

    let cookie = jar.get("user_id");

    let id = match cookie {
        Some(x) => x,
        None => return Err(()),
    };

    let id: i64 = id.value().parse().unwrap();

    let message = Message::from_newmsg(message.into_inner(), id);
    let x = crate::database::messages::send_message(&mut conn, message)
        .await
        .unwrap();

    Ok(MessageId { 0: x }.into())
}

///TODO check that the user has reading privilage in the given room
#[post("/get_message", data = "<message_id>")]
async fn get_message(
    mut conn: Connection<DbConn>,
    jar: &CookieJar<'_>,
    message_id: Json<MessageId>,
) -> Result<Json<Message>, ()> {
    let cookie = jar.get("user_id");

    let user_id = match cookie {
        Some(x) => x,
        None => return Err(()),
    };

    let _user_id: i64 = user_id.value().parse().unwrap();

    let message = crate::database::messages::get_msg_by_id(&mut conn, message_id.into_inner().0)
        .await
        .unwrap();

    match message {
        Some(x) => Ok(x.into()),
        None => Err(()),
    }
}

pub fn routes() -> Vec<Route> {
    routes![send_message, get_message]
}
