// use std::fmt::Display;

use crate::{
    db_event_types::{RoomEvent, RoomEventType},
    db_types::{Community, Message, Room, ServerMember},
    // schema::db_schema::{self, community_members, room_events, rooms},
    transmission::{AuthErr, InsertResult, JoinServerResult, UserAuth},
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
// use rocket::serde::Deserialize;
// use rocket_db_pools::{Connection, Database};
use rocket_db_pools::Connection;

pub struct User {
    pub id: Option<i64>,
    pub username: String,
    pub nickname: Option<String>,
    password: String,
}

impl User {
    pub async fn insert(new_user: UserAuth, conn: &mut Connection<DbConn>) -> InsertResult {
        let result = has_user(&mut ***conn, new_user.username.clone()).await;
        if result.is_err() {
            return InsertResult::DbError;
        }
        if result.unwrap() {
            return InsertResult::UsernameTaken;
        }

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(new_user.password.as_bytes(), &salt);

        if password_hash.is_err() {
            return InsertResult::InvalidPassword;
        }

        let pass = password_hash.unwrap().to_string();

        let t = User {
            id: None,
            username: new_user.username,
            nickname: None,
            password: pass,
        };

        let e = insert_user(conn, t).await;
        match e {
            Ok(x) => {
                return InsertResult::Success(x);
            }
            Err(_x) => {
                println!("insert");
                dbg!(_x);
                return InsertResult::DbError;
            }
        };
    }

    pub async fn auth(user: UserAuth, conn: &mut Connection<DbConn>) -> AuthErr {
        let e = get_user_by_name(conn, user.username).await;
        let query = e.unwrap();

        if query.is_none() {
            return AuthErr::InvalidUsername;
        }

        let query = query.unwrap();

        let password_hash = PasswordHash::new(&query.password).unwrap();
        let verified = Argon2::default().verify_password(user.password.as_bytes(), &password_hash);

        match verified {
            Ok(_) => AuthErr::Success(query.id.unwrap()),
            Err(_) => AuthErr::InvalidPassword,
        }
    }
}

#[derive(rocket_db_pools::Database)]
#[database("sqlx")]
pub struct DbConn(sqlx::PgPool);

use rocket_db_pools::Initializer;
use sqlx::{Error, PgConnection};

impl DbConn {
    pub fn init() -> Initializer<Self> {
        Initializer::new()
    }
}

pub async fn get_user_by_id(conn: &mut Connection<DbConn>, id: i64) -> Result<Option<User>, Error> {
    sqlx::query_as!(User, "select * from users where id = $1", id)
        .fetch_optional(&mut ***conn)
        .await
}

pub async fn get_user_by_name(
    conn: &mut Connection<DbConn>,
    name: String,
) -> Result<Option<User>, Error> {
    sqlx::query_as!(User, "select * from users where username = $1", name)
        .fetch_optional(&mut ***conn)
        .await
}

pub async fn get_user_name(conn: &mut Connection<DbConn>, id: i64) -> Result<String, Error> {
    let user = sqlx::query_as!(User, "select * from users where id = $1", id)
        .fetch_optional(&mut ***conn)
        .await;

    match user {
        Ok(x) => Ok(x.unwrap().username),
        Err(x) => Err(x),
    }
}

/// inserts a user and returns their id
pub async fn insert_user(conn: &mut Connection<DbConn>, user: User) -> Result<i64, Error> {
    let result = sqlx::query!(
        "INSERT INTO users(username, nickname, password) VALUES($1, $2, $3) RETURNING id",
        user.username,
        user.nickname,
        user.password
    )
    .fetch_one(&mut ***conn)
    .await;

    match result {
        Ok(x) => Ok(x.id),
        Err(x) => Err(x),
    }
}

pub async fn has_user(conn: &mut PgConnection, name: String) -> Result<bool, Error> {
    let user = sqlx::query_as!(User, "select * from users where username = $1", name)
        .fetch_optional(conn)
        .await;

    match user {
        Ok(x) => Ok(x.is_some()),
        Err(x) => Err(x),
    }
}

pub async fn get_msg_by_id(
    conn: &mut Connection<DbConn>,
    id: i64,
) -> Result<Option<Message>, Error> {
    let user = sqlx::query_as!(Message, "select * from messages where id = $1", id)
        .fetch_optional(&mut ***conn)
        .await;

    match user {
        Ok(x) => Ok(x),
        Err(x) => Err(x),
    }
}

pub async fn send_message(conn: &mut Connection<DbConn>, message: Message) -> Result<i64, Error> {
    let timestamp = message.timestamp;
    let channel_id = message.channel;
    let server_id = message.server;

    let result = sqlx::query!(
        "INSERT INTO messages(sender, server, channel, reply, is_reply, text, timestamp) VALUES($1, $2, $3, $4, $5, $6, $7) RETURNING id",
        message.sender, message.server, message.channel, message.reply, message.is_reply, message.text, message.timestamp
    ).fetch_one(&mut ***conn)
    .await;

    match result {
        Ok(x) => {
            let _y = create_channel_event(
                conn,
                channel_id,
                server_id,
                timestamp,
                RoomEventType::NewMessage(x.id),
            )
            .await;

            return Ok(x.id);
        }
        Err(x) => return Err(x),
    }
}

pub async fn get_community_members(
    conn: &mut Connection<DbConn>,
    server_id: i64,
) -> Result<Vec<ServerMember>, Error> {
    let mut val = sqlx::query_as!(
        ServerMember,
        "SELECT * FROM community_members WHERE server_id = $1",
        server_id
    )
    .fetch_all(&mut ***conn)
    .await;

    match &mut val {
        Ok(y) => {
            for member in y {
                if member.nickname.is_none() {
                    let uname = get_user_name(conn, member.userid).await;
                    member.nickname = Some(uname.unwrap_or("unable to fetch".to_string()));
                }
            }
        }
        Err(x) => {
            println!("err in get server members");
            dbg!(&x);
        }
    }

    val
}

/// gets all servers a user is a part of
pub async fn get_user_communities(
    conn: &mut Connection<DbConn>,
    uid: i64,
) -> Result<Vec<ServerMember>, Error> {
    sqlx::query_as!(
        ServerMember,
        "SELECT * FROM community_members WHERE userid = $1",
        uid
    )
    .fetch_all(&mut ***conn)
    .await
}

/// gets all rooms in a community
pub async fn get_community_rooms(
    conn: &mut Connection<DbConn>,
    server_id: i64,
) -> Result<Vec<Room>, Error> {
    let a = sqlx::query_as!(Room, "SELECT * FROM rooms WHERE server = $1", server_id)
        .fetch_all(&mut ***conn)
        .await;

    a
}

pub async fn join_community(
    conn: &mut Connection<DbConn>,
    server_id: i64,
    userid: i64,
    nickname: Option<String>,
) -> JoinServerResult {
    let _result = sqlx::query!(
        "INSERT INTO community_members(server_id, userid, nickname) VALUES($1, $2, $3)",
        server_id,
        userid,
        nickname
    )
    .fetch_one(&mut ***conn)
    .await;

    JoinServerResult::Success(server_id)
}

pub async fn create_channel_event(
    conn: &mut Connection<DbConn>,
    channel_id: i64,
    server_id: i64,
    timestamp: i64,
    event_type: RoomEventType,
) -> Result<i64, Error> {
    let event = event_type.to_event(channel_id, server_id, timestamp);

    let result = sqlx::query!(
        "INSERT INTO room_events(channel_id, server_id, timestamp, event_type, message, reaction, creator, deleted) VALUES($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
        event.channel_id, event.server_id, event.timestamp, event.event_type, event.message, event.reaction, event.creator, event.deleted
    ).fetch_one(&mut ***conn)
    .await;

    match result {
        Ok(x) => return Ok(x.id),
        Err(x) => return Err(x),
    }
}

/// gets all events in a room that have happened after the provided timestamp excluding the message with the provided id
pub async fn get_room_events_since_timestamp_and_id(
    conn: &mut Connection<DbConn>,
    channel_id: i64,
    since: i64,
    id: i64,
    amount: i64,
) -> Result<Vec<RoomEvent>, Error> {
    let mut a = sqlx::query_as!(
        RoomEvent,
        "SELECT * FROM room_events WHERE channel_id = $1 AND timestamp >= $2 AND id != $3 ORDER BY timestamp DESC LIMIT $4",
        channel_id, since, id, amount
    ).fetch_all(&mut ***conn).await;

    match &mut a {
        Ok(x) => {
            x.sort_unstable_by_key(|y| y.timestamp);
        }
        Err(_) => {}
    }

    a
}

/// get events that happened prior to a given event
pub async fn get_events_prior(
    conn: &mut Connection<DbConn>,
    channel_id: i64,
    prior_to: i64,
    last_msg: i64,
    amount: i64,
) -> Result<Vec<RoomEvent>, Error> {
    let mut a = sqlx::query_as!(
        RoomEvent,
        "SELECT * FROM room_events WHERE channel_id = $1 AND timestamp <= $2 AND id != $3 ORDER BY timestamp ASC LIMIT $4",
        channel_id, prior_to, last_msg, amount
    ).fetch_all(&mut ***conn).await;

    match &mut a {
        Ok(x) => {
            x.sort_unstable_by_key(|y| y.timestamp);
        }
        Err(_) => {}
    }

    a
}

/// get the latest n events in a room, good for first opening a room
pub async fn get_room_events(
    conn: &mut Connection<DbConn>,
    channel_id: i64,
    amount: i64,
) -> Result<Vec<RoomEvent>, Error> {
    let mut a = sqlx::query_as!(
        RoomEvent,
        "SELECT * FROM room_events WHERE channel_id = $1 ORDER BY timestamp DESC LIMIT $2",
        channel_id,
        amount
    )
    .fetch_all(&mut ***conn)
    .await;

    match &mut a {
        Ok(x) => {
            x.sort_unstable_by_key(|y| y.timestamp);
        }
        Err(_) => {}
    }

    a
}

/// creates a community and a general room and returns the community id
pub async fn create_community(
    conn: &mut Connection<DbConn>,
    creator: i64,
    name: String,
) -> Result<i64, Error> {
    let result = sqlx::query!(
        "INSERT INTO communities(nickname, owner, is_public) VALUES($1, $2, $3) RETURNING id",
        name, creator, false
    ).fetch_one(&mut ***conn)
    .await;

    let id;
    match result {
        Ok(x) => id = x.id,
        Err(x) => return Err(x),
    }

    let _x = join_community(conn, id, creator, None).await;

    //it is ok to ignore this as it is not a critical error if the general room failed to be created because of a disconnect to the db
    //if a user were to create a community and right before this line the connection breaks it would simply create a community with no rooms
    //and the user could just create a general room manually once the database has been fixed
    let _result = create_room(conn, creator, id, "general".to_string()).await;

    return Ok(id);
}

pub async fn is_admin(
    conn: &mut Connection<DbConn>,
    userid: i64,
    community: i64,
) -> Result<bool, Error> {
    let a = sqlx::query!(
        "SELECT owner as id FROM communities WHERE id = $1",
        community
    )
    .fetch_optional(&mut ***conn)
    .await;

    match a {
        Ok(x) => {
            if x.is_some_and(|x| x.id == Some(userid)) {
                return Ok(true);
            }
        },
        Err(x) => return Err(x),
    }

    let a = sqlx::query!(
        "SELECT * FROM roles JOIN role_members on roles.id = role_members.roleid WHERE community = $1 AND is_admin = true AND userid = $2 LIMIT 1",
        community, userid
    )
    .fetch_optional(&mut ***conn)
    .await;

    match a {
        Ok(x) => {
            if x.is_some() {
                return Ok(true);
            }
        },
        Err(_x) => {},
    }

    return Ok(false);
}

pub enum CreateRoomResult {
    Success(i64),
    Failure,
    NotAuthorised,
    InvalidName,
    NameTaken,
}
pub async fn create_room(
    conn: &mut Connection<DbConn>,
    creator: i64,
    community: i64,
    name: String,
) -> Result<CreateRoomResult, Error> {
    let admin = is_admin(conn, creator, community).await;

    match admin {
        Ok(x) => {
            if !x {
                return Ok(CreateRoomResult::NotAuthorised);
            }
        },
        Err(x) => return Err(x),
    }

    let result = sqlx::query!(
        "SELECT * FROM rooms WHERE server = $1 AND name = $2 LIMIT 1",
        community, name
    ).fetch_optional(&mut ***conn)
    .await;

    if result.is_err() {
        return Err(result.unwrap_err());
    }

    if result.is_ok_and(|x| x.is_some()) {
        return Ok(CreateRoomResult::NameTaken);
    }

    let result = sqlx::query!(
        "INSERT INTO rooms(server, name) VALUES($1, $2) RETURNING id",
        community, name
    ).fetch_one(&mut ***conn)
    .await;

    match result {
        Ok(x) => Ok(CreateRoomResult::Success(x.id)),
        Err(x) => Err(x),
    }
}
