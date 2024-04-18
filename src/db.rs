use std::fmt::Display;

use crate::{
    db_event_types::{RoomEvent, RoomEventType},
    db_types::{Message, Room, ServerMember},
    // schema::db_schema::{self, community_members, room_events, rooms},
    transmission::{AuthErr, InsertError, JoinServerResult, UserAuth},
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
// use rocket::serde::Deserialize;
use rocket_db_pools::{Connection, Database};

// use ormx::{Insert, Table, Delete};

// #[derive(Deserialize, Queryable, Insertable, Debug)]
// #[diesel(table_name = db_schema::users)]
// #[derive(Debug, ormx::Table)]
// #[ormx(table = "users", id = user_id, insertable, deletable)]
// #[derive(Clone, Debug, PartialEq, DeriveEntity)]
// #[sea_orm(table_name = "users")]
pub struct User {
    // #[sea_orm(primary_key)]
    pub id: Option<i64>,
    pub username: String,
    pub nickname: Option<String>,
    password: String,
}

impl User {
    pub async fn insert(new_user: UserAuth, conn: &mut Connection<DbConn>) -> InsertError {
        if has_user(&mut ***conn, new_user.username.clone()).await {
            return InsertError::UsernameTaken;
        }

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(new_user.password.as_bytes(), &salt);

        if password_hash.is_err() {
            return InsertError::InvalidPassword;
        }

        let pass = password_hash.unwrap().to_string();

        let t = User {
            id: None,
            username: new_user.username,
            nickname: None,
            password: pass,
        };

        let e = insert_user(&mut ***conn, t).await;
        match e {
            Ok(x) => {return InsertError::Success(x);},
            Err(_x) => {
                println!("insert");
                dbg!(_x);
                return InsertError::DbError;
            }
        };
    }

    pub async fn auth(user: UserAuth, conn: &mut PgConnection) -> AuthErr {
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

// use rocket_db_pools::{Database, Connection};

// #[database("diesel")]
// pub struct DbConn(diesel::PgConnection);
#[derive(rocket_db_pools::Database)]
#[database("sqlx")]
pub struct DbConn(sqlx::PgPool);

// use db_schema::{
//     // messages::{self, channel},
//     messages::{self},
//     // messages::self,
//     // usernames, users,
//     users,
// };
// use diesel::{prelude::*, result::Error, sql_types::Integer};

// #[derive(QueryableByName)]
// pub struct InsertedRowId {
//     #[diesel(sql_type = Integer)]
//     pub id: i32,
// }

use rocket_db_pools::Initializer;
// use sea_orm::{query, DeriveEntity, DeriveEntityModel};
use sqlx::{Error, PgConnection};

impl DbConn {
    pub fn init() -> Initializer<Self> {
        Initializer::new()
    }
}

// impl DbConn {

//     pub fn init() -> Initializer<Self> {
//         Initializer::new()
//     }

// pub async fn get_user_by_id(&self, id: i64) -> Result<User, Error> {
//     // let user: User = self
//     //     .run(move |conn| users::table.filter(users::id.eq(id)).first(conn))
//     //     .await?;
//     let user: User = sqlx::query_as!(User, "SELECT DISTINCT FROM ");
//     Ok(user)
// }

pub async fn get_user_by_id(conn: &Connection<DbConn>, id: i64) -> Result<User, Error> {
    todo!()
}

pub async fn get_user_by_name(conn: &mut PgConnection, name: String) -> Result<Option<User>, Error> {

    let user = sqlx::query_as!(
        User,
        "select * from users where username = $1",
        name
    )
    .fetch_optional(conn)
    .await;

    return user;
}

// pub async fn get_user_name(&self, id: i64) -> Result<String, Error> {
//     let user: User = self
//         .run(move |conn| {
//             db_schema::users::table
//                 .filter(db_schema::users::id.eq(id))
//                 .first(conn)
//         })
//         .await?;
//     Ok(user.username)
// }

pub async fn get_user_name(conn: &Connection<DbConn>, id: i64) -> Result<String, Error> {
    todo!()
}

pub async fn insert_user(conn: &mut PgConnection, user: User) -> Result<usize, Error> {
    todo!()
}

// pub async fn insert_user(&self, user: User) -> Result<usize, Error> {
//     self.run(move |c| {
//         diesel::insert_into(db_schema::users::table)
//             .values(user)
//             .execute(c)
//     })
//     .await
// }

pub async fn has_user(conn: &mut PgConnection, name: String) -> bool {
    todo!()
}

// pub async fn has_user(&self, name: String) -> bool {
//     let e = self.get_user_by_name(name).await;
//     e.is_ok()
// }

pub async fn get_msg_by_id(conn: &Connection<DbConn>, id: i64) -> Result<Message, Error> {
    todo!()
}

// pub async fn get_msg_by_id(&self, id: i64) -> Result<Message, Error> {
//     let message: Message = self
//         .run(move |conn| messages::table.filter(messages::id.eq(id)).first(conn))
//         .await?;
//     Ok(message)
// }

pub async fn send_message(conn: &Connection<DbConn>, message: Message) -> Result<Message, Error> {
    todo!()
}

// pub async fn send_message(&self, message: Message) -> Result<Message, diesel::result::Error> {
//     let timestamp = message.timestamp;
//     let channel_id = message.channel;
//     let server_id = message.server;

//     let err_t = self
//         .run(move |c| {
//             c.transaction(|c| {
//                 let _a = diesel::insert_into(db_schema::messages::table)
//                     .values(message)
//                     .get_result::<Message>(c);
//                 diesel::result::QueryResult::Ok(_a)
//             })
//         })
//         .await?;

//     match &err_t {
//         Ok(x) => {
//             println!("inserting {}", x.id.unwrap());
//             let y = self
//                 .create_channel_event(
//                     channel_id,
//                     server_id,
//                     timestamp,
//                     RoomEventType::NewMessage(x.id.unwrap()),
//                 )
//                 .await;
//             dbg!(&y);
//             let _restut = y.expect("unable to insert message event for new message");
//         }
//         Err(_) => todo!(),
//     }

//     err_t

//     // return Ok(1);
// }

pub async fn get_community_members(
    conn: &Connection<DbConn>,
    server_id: i64,
) -> Result<Vec<ServerMember>, Error> {
    todo!()
}

// pub async fn get_community_members(
//     &self,
//     server_id: i64,
// ) -> Result<Vec<ServerMember>, diesel::result::Error> {
//     let mut val = self
//         .run(move |conn| {
//             community_members::dsl::community_members
//                 .filter(community_members::dsl::server_id.eq(server_id))
//                 .load::<ServerMember>(conn)
//         })
//         .await;

//     match &mut val {
//         Ok(y) => {
//             for member in y {
//                 if member.nickname.is_none() {
//                     let uname = self.get_user_name(member.userid).await;
//                     member.nickname = Some(uname.unwrap_or("unable to fetch".to_string()));
//                 }
//             }
//         }
//         Err(x) => {
//             println!("err in get server members");
//             dbg!(&x);
//         }
//     }

//     val
// }

pub async fn get_user_communities(
    conn: &Connection<DbConn>,
    uid: i64,
) -> Result<Vec<ServerMember>, Error> {
    todo!()
}

//gets all servers a user is a part of
// pub async fn get_user_communities(
//     &self,
//     uid: i64,
// ) -> Result<Vec<ServerMember>, diesel::result::Error> {
//     self.run(move |conn| {
//         community_members::dsl::community_members
//             .filter(community_members::dsl::userid.eq(uid))
//             .load::<ServerMember>(conn)
//     })
//     .await
// }

pub async fn get_community_rooms(
    conn: &Connection<DbConn>,
    server_id: i64,
) -> Result<Vec<Room>, ()> {
    todo!()
}

// pub async fn get_community_rooms(
//     &self,
//     server_id: i64,
// ) -> Result<Vec<Room>, diesel::result::Error> {
//     self.run(move |conn| {
//         rooms::dsl::rooms
//             .filter(rooms::dsl::server.eq(server_id))
//             .load::<Room>(conn)
//     })
//     .await
// }

pub async fn join_community(
    conn: &Connection<DbConn>,
    server_id: i64,
    userid: i64,
    nickname: Option<String>,
) -> JoinServerResult {
    todo!()
}

// pub async fn join_community(
//     &self,
//     server_id: i64,
//     userid: i64,
//     nickname: Option<String>,
// ) -> JoinServerResult {
//     let message: ServerMember = ServerMember {
//         server_id,
//         userid,
//         nickname,
//     };
//     let e = self
//         .run(move |c| {
//             diesel::insert_into(db_schema::community_members::table)
//                 .values(message)
//                 .execute(c)
//         })
//         .await;

//     match e {
//         Ok(x) => JoinServerResult::Success(x.try_into().unwrap()),
//         Err(x) => {
//             dbg!(x);
//             JoinServerResult::AlreadyInServer
//         }
//     }
// }

pub async fn create_channel_event(
    conn: &Connection<DbConn>,
    channel_id: i64,
    server_id: i64,
    timestamp: i64,
    event_type: RoomEventType,
) -> Result<usize, ()> {
    todo!()
}

// pub async fn create_channel_event(
//     &self,
//     channel_id: i64,
//     server_id: i64,
//     timestamp: i64,
//     event_type: RoomEventType,
// ) -> Result<usize, diesel::result::Error> {
//     let event = event_type.to_event(channel_id, server_id, timestamp);

//     self.run(move |c| {
//         diesel::insert_into(db_schema::room_events::table)
//             .values(event)
//             .execute(c)
//     })
//     .await
// }

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

// pub async fn get_room_events_since_timestamp_and_id(
//     &self,
//     channel_id: i64,
//     since: i64,
//     id: i64,
//     amount: i64,
// ) -> Result<Vec<RoomEvent>, diesel::result::Error> {
//     let mut a = self
//         .run(move |conn| {
//             room_events::dsl::room_events
//                 .filter(room_events::dsl::channel_id.eq(channel_id))
//                 .filter(room_events::dsl::timestamp.ge(since))
//                 .order(room_events::dsl::timestamp.desc())
//                 .limit(amount)
//                 .filter(room_events::dsl::id.ne(id))
//                 .load::<RoomEvent>(conn)
//         })
//         .await;

//     match &mut a {
//         Ok(x) => {
//             x.sort_unstable_by_key(|y| y.timestamp);
//         }
//         Err(_) => {}
//     }

//     a
// }

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

// pub async fn get_events_prior(
//     &self,
//     // server_id: i32,
//     channel_id: i64,
//     prior_to: i64,
//     last_msg: i64,
//     amount: i64,
// ) -> Result<Vec<RoomEvent>, Error> {
//     let mut val = self
//         .run(move |conn| {
//             room_events::dsl::room_events
//                 // .filter(messages::dsl::server.eq(server_id))
//                 .filter(room_events::dsl::channel_id.eq(channel_id))
//                 .filter(room_events::dsl::timestamp.le(prior_to))
//                 .filter(room_events::dsl::id.ne(last_msg))
//                 .order(room_events::dsl::timestamp.asc())
//                 .limit(amount)
//                 // .order(messages::dsl::id.desc())
//                 // .order(messages::dsl::timestamp.asc())
//                 .load::<RoomEvent>(conn)
//             // .order(messages::dsl::timestamp.asc())
//         })
//         .await;

//     match &mut val {
//         Ok(x) => {
//             x.sort_unstable_by_key(|y| y.timestamp);
//         }
//         Err(_) => {}
//     }

//     val
// }

pub async fn get_channel_events(
    conn: &mut Connection<DbConn>,
    channel_id: i64,
    amount: i64,
) -> Result<Vec<RoomEvent>, Error> {
    let mut a = sqlx::query_as!(
        RoomEvent,
        "SELECT * FROM room_events WHERE channel_id = $1 ORDER BY timestamp DESC LIMIT $2",
        channel_id, amount
    ).fetch_all(&mut ***conn).await;

    match &mut a {
        Ok(x) => {
            x.sort_unstable_by_key(|y| y.timestamp);
        }
        Err(_) => {}
    }

    a
}

// pub async fn get_channel_events(
//     &self,
//     // server_id: i32,
//     channel_id: i64,
//     amount: i64,
// ) -> Result<Vec<RoomEvent>, Error> {
//     let mut val = self
//         .run(move |conn| {
//             room_events::dsl::room_events
//                 // .filter(messages::dsl::server.eq(server_id))
//                 .filter(room_events::dsl::channel_id.eq(channel_id))
//                 .order(room_events::dsl::timestamp.desc())
//                 .limit(amount)
//                 // .order(messages::dsl::id.desc())
//                 // .order(messages::dsl::timestamp.asc())
//                 .load::<RoomEvent>(conn)
//             // .order(messages::dsl::timestamp.asc())
//         })
//         .await;

//     match &mut val {
//         Ok(x) => {
//             x.sort_unstable_by_key(|y| y.timestamp);
//         }
//         Err(_) => {}
//     }

//     val
// }
// }
