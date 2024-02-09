use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
// use chrono::NaiveDateTime;
use crate::schema::db_schema::{self, channels, server_members};
use rocket::serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserAuth {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum InsertError {
    Success(usize),
    UsernameTaken,
    DbError,
    InvalidPassword,
    InvalidUsername,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum AuthErr {
    Success(i32),
    InvalidUsername,
    InvalidPassword,
}

#[derive(Deserialize, Queryable, Insertable, Debug)]
#[diesel(table_name = db_schema::users)]
pub struct User {
    pub id: Option<i32>,
    pub username: String,
    pub nickname: Option<String>,
    password: String,
}

// #[derive(Deserialize, Queryable, Insertable, Debug)]
// #[diesel(table_name = schema::usernames)]
// pub struct UsernameMap {
//     pub userid: i32,
//     pub username: String,
// }

impl User {
    pub async fn insert(new_user: UserAuth, conn: &DbConn) -> InsertError {
        if conn.has_user(new_user.username.clone()).await {
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

        let e = conn.insert_user(t).await;
        match e {
            Ok(x) => InsertError::Success(x),
            Err(_x) => {
                println!("insert");
                dbg!(_x);
                InsertError::DbError
            }
        }
    }

    pub async fn auth(user: UserAuth, conn: &DbConn) -> AuthErr {
        let e = conn.get_user_by_name(user.username).await;

        if e.is_err() {
            return AuthErr::InvalidUsername;
        }

        let query = e.unwrap();

        let password_hash = PasswordHash::new(&query.password).unwrap();
        let verified = Argon2::default().verify_password(user.password.as_bytes(), &password_hash);

        match verified {
            Ok(_) => AuthErr::Success(query.id.unwrap()),
            Err(_) => AuthErr::InvalidPassword,
        }
    }
}

#[derive(Deserialize, Queryable, Insertable, Debug, Serialize, Clone)]
#[diesel(table_name = db_schema::messages)]
pub struct Message {
    pub id: Option<i32>,
    pub sender: i32,
    pub server: i32,
    pub channel: i32,
    pub reply: Option<i32>,
    pub text: String,
    // pub emoji: Option<Vec<u8>>,
    pub timestamp: i64,
}

#[derive(Deserialize, Queryable, Insertable, Debug, Serialize, Clone)]
#[diesel(table_name = db_schema::server_members)]
pub struct ServerMember {
    // pub id: Option<i32>,
    pub server_id: i32,
    pub userid: i32,
    pub nickname: Option<String>,
}

#[derive(Deserialize, Queryable, Insertable, Debug, Serialize, Clone)]
#[diesel(table_name = db_schema::channels)]
pub struct Channel {
    pub id: Option<i32>,
    pub server: i32,
    pub name: String,
}

#[derive(Deserialize, Queryable, Insertable, Debug, Serialize, Clone)]
#[diesel(table_name = db_schema::channel_events)]
pub struct ChannelEvent {
    pub id: Option<i32>,
    pub channel_id: i32,
    pub timestamp: i64,
    pub event_type: i32,
    pub message: Option<i32>,
    pub reaction: Option<i32>,
    pub user: Option<i32>,
    pub deleted: Option<i32>, //used for the id of deleted content
}

pub enum ChannelEventType {
    NewMessage(i32),
    MessageDeleted(i32),
    NewReaction(i32),
    DeleteReaction(i32),
    UserJoin(i32),
    UserLeave(i32),
    Error,
}

impl ChannelEvent {
    fn to_event_type(&self) -> ChannelEventType {
        match self.event_type {
            0 => ChannelEventType::NewMessage(self.message.unwrap()),
            1 => ChannelEventType::MessageDeleted(self.deleted.unwrap()),
            2 => ChannelEventType::NewReaction(self.reaction.unwrap()),
            3 => ChannelEventType::DeleteReaction(self.deleted.unwrap()),
            4 => ChannelEventType::UserJoin(self.user.unwrap()),
            5 => ChannelEventType::UserLeave(self.user.unwrap()),
            _ => ChannelEventType::Error,
        }
    }
}

impl ChannelEventType {
    fn to_int(&self) -> i32 {
        match self {
            ChannelEventType::NewMessage(_) => 0,
            ChannelEventType::MessageDeleted(_) => 1,
            ChannelEventType::NewReaction(_) => 2,
            ChannelEventType::DeleteReaction(_) => 3,
            ChannelEventType::UserJoin(_) => 4,
            ChannelEventType::UserLeave(_) => 5,
            ChannelEventType::Error => -1,
        }
    }
    fn to_event(&self, channel_id: i32, timestamp: i64) -> ChannelEvent {
        match self {
            ChannelEventType::NewMessage(x) => ChannelEvent {
                id: None,
                channel_id,
                timestamp,
                event_type: self.to_int(),
                message: Some(*x),
                reaction: None,
                user: None,
                deleted: None,
            },
            ChannelEventType::MessageDeleted(x) => ChannelEvent {
                id: None,
                channel_id,
                timestamp,
                event_type: self.to_int(),
                message: None,
                reaction: None,
                user: None,
                deleted: Some(*x),
            },
            ChannelEventType::NewReaction(x) => ChannelEvent {
                id: None,
                channel_id,
                timestamp,
                event_type: self.to_int(),
                message: None,
                reaction: Some(*x),
                user: None,
                deleted: None,
            },
            ChannelEventType::DeleteReaction(x) => ChannelEvent {
                id: None,
                channel_id,
                timestamp,
                event_type: self.to_int(),
                message: None,
                reaction: None,
                user: None,
                deleted: Some(*x),
            },
            ChannelEventType::UserJoin(x) => ChannelEvent {
                id: None,
                channel_id,
                timestamp,
                event_type: self.to_int(),
                message: None,
                reaction: None,
                user: Some(*x),
                deleted: None,
            },
            ChannelEventType::UserLeave(x) => ChannelEvent {
                id: None,
                channel_id,
                timestamp,
                event_type: self.to_int(),
                message: None,
                reaction: None,
                user: Some(*x),
                deleted: None,
            },
            ChannelEventType::Error => ChannelEvent {
                id: None,
                channel_id,
                timestamp,
                event_type: self.to_int(),
                message: None,
                reaction: None,
                user: None,
                deleted: None,
            },
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum JoinServerResult {
    Success(i32),
    AlreadyInServer,
    NotAuthorised,
    Failure,
}

#[database("diesel")]
pub struct DbConn(diesel::SqliteConnection);
use db_schema::{
    // messages::{self, channel},
    messages::{self},
    // messages::self,
    // usernames, users,
    users,
};
use diesel::{prelude::*, result::Error};

impl DbConn {
    pub async fn get_user_by_id(&self, id: i32) -> Result<User, Error> {
        let user: User = self
            .run(move |conn| users::table.filter(users::id.eq(id)).first(conn))
            .await?;
        Ok(user)
    }

    pub async fn get_user_by_name(&self, name: String) -> Result<User, Error> {
        let user: User = self
            .run(move |conn| users::table.filter(users::username.eq(name)).first(conn))
            .await?;
        Ok(user)
    }

    pub async fn get_user_name(&self, id: i32) -> Result<String, Error> {
        let user: User = self
            .run(move |conn| {
                db_schema::users::table
                    .filter(db_schema::users::id.eq(id))
                    .first(conn)
            })
            .await?;
        Ok(user.username)
    }

    pub async fn insert_user(&self, user: User) -> Result<usize, Error> {
        self.run(move |c| {
            diesel::insert_into(db_schema::users::table)
                .values(user)
                .execute(c)
        })
        .await
    }

    pub async fn has_user(&self, name: String) -> bool {
        let e = self.get_user_by_name(name).await;
        e.is_ok()
    }

    pub async fn get_msg_by_id(&self, id: i32) -> Result<Message, Error> {
        let message: Message = self
            .run(move |conn| messages::table.filter(messages::id.eq(id)).first(conn))
            .await?;
        Ok(message)
    }

    pub async fn send_message(&self, message: Message) -> Result<usize, diesel::result::Error> {
        self.run(move |c| {
            diesel::insert_into(db_schema::messages::table)
                .values(message)
                .execute(c)
        })
        .await
    }

    pub async fn get_channel_messages(
        &self,
        server_id: i32,
        channel_id: i32,
        amount: i64,
    ) -> Result<Vec<Message>, Error> {
        let mut val = self
            .run(move |conn| {
                messages::dsl::messages
                    .filter(messages::dsl::server.eq(server_id))
                    .filter(messages::dsl::channel.eq(channel_id))
                    .order(messages::dsl::timestamp.desc())
                    .limit(amount)
                    // .order(messages::dsl::id.desc())
                    // .order(messages::dsl::timestamp.asc())
                    .load::<Message>(conn)
                // .order(messages::dsl::timestamp.asc())
            })
            .await;

        match &mut val {
            Ok(x) => {
                x.sort_unstable_by_key(|y| y.timestamp);
            }
            Err(_) => {}
        }

        val
    }

    pub async fn get_messages_prior(
        &self,
        server_id: i32,
        channel_id: i32,
        prior_to: i64,
        last_msg: i32,
        amount: i64,
    ) -> Result<Vec<Message>, Error> {
        let mut val = self
            .run(move |conn| {
                messages::dsl::messages
                    .filter(messages::dsl::server.eq(server_id))
                    .filter(messages::dsl::channel.eq(channel_id))
                    .filter(messages::dsl::timestamp.le(prior_to))
                    .filter(messages::dsl::id.ne(last_msg))
                    .order(messages::dsl::timestamp.asc())
                    .limit(amount)
                    // .order(messages::dsl::id.desc())
                    // .order(messages::dsl::timestamp.asc())
                    .load::<Message>(conn)
                // .order(messages::dsl::timestamp.asc())
            })
            .await;

        match &mut val {
            Ok(x) => {
                x.sort_unstable_by_key(|y| y.timestamp);
            }
            Err(_) => {}
        }

        val
    }

    pub async fn get_messages_since_dt(
        &self,
        server_id: i32,
        channel_id: i32,
        since: chrono::NaiveDateTime,
        amount: i64,
    ) -> Result<Vec<Message>, diesel::result::Error> {
        self.run(move |conn| {
            messages::dsl::messages
                .filter(messages::dsl::server.eq(server_id))
                .filter(messages::dsl::channel.eq(channel_id))
                .filter(messages::dsl::timestamp.ge(since.timestamp_millis()))
                .order(messages::dsl::timestamp.desc())
                .limit(amount)
                .order(messages::dsl::timestamp.asc())
                // .order(messages::dsl::id.desc())
                .load::<Message>(conn)
        })
        .await
    }

    pub async fn get_messages_since_timestamp(
        &self,
        server_id: i32,
        channel_id: i32,
        since: i64,
        amount: i64,
    ) -> Result<Vec<Message>, diesel::result::Error> {
        self.run(move |conn| {
            messages::dsl::messages
                .filter(messages::dsl::server.eq(server_id))
                .filter(messages::dsl::channel.eq(channel_id))
                .filter(messages::dsl::timestamp.ge(since))
                .order(messages::dsl::timestamp.desc())
                .limit(amount)
                .order(messages::dsl::timestamp.asc())
                // .order(messages::dsl::id.desc())
                .load::<Message>(conn)
        })
        .await
    }

    pub async fn get_messages_since_timestamp_and_id(
        &self,
        server_id: i32,
        channel_id: i32,
        since: i64,
        id: i32,
        amount: i64,
    ) -> Result<Vec<Message>, diesel::result::Error> {
        self.run(move |conn| {
            messages::dsl::messages
                .filter(messages::dsl::server.eq(server_id))
                .filter(messages::dsl::channel.eq(channel_id))
                .filter(messages::dsl::timestamp.ge(since))
                .order(messages::dsl::timestamp.desc())
                .limit(amount)
                // .order(messages::dsl::id.desc())
                .order(messages::dsl::timestamp.asc())
                .filter(messages::dsl::id.ne(id))
                .load::<Message>(conn)
        })
        .await
    }

    pub async fn get_server_members(
        &self,
        server_id: i32,
    ) -> Result<Vec<ServerMember>, diesel::result::Error> {
        let mut val = self
            .run(move |conn| {
                server_members::dsl::server_members
                    .filter(server_members::dsl::server_id.eq(server_id))
                    .load::<ServerMember>(conn)
            })
            .await;

        match &mut val {
            Ok(y) => {
                for member in y {
                    if member.nickname.is_none() {
                        let uname = self.get_user_name(member.userid).await;
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

    //gets all servers a user is a part of
    pub async fn get_user_servers(
        &self,
        uid: i32,
    ) -> Result<Vec<ServerMember>, diesel::result::Error> {
        self.run(move |conn| {
            server_members::dsl::server_members
                .filter(server_members::dsl::userid.eq(uid))
                .load::<ServerMember>(conn)
        })
        .await
    }

    pub async fn get_server_channels(
        &self,
        server_id: i32,
    ) -> Result<Vec<Channel>, diesel::result::Error> {
        self.run(move |conn| {
            channels::dsl::channels
                .filter(channels::dsl::server.eq(server_id))
                .load::<Channel>(conn)
        })
        .await
    }

    pub async fn join_server(&self, message: ServerMember) -> JoinServerResult {
        let e = self
            .run(move |c| {
                diesel::insert_into(db_schema::server_members::table)
                    .values(message)
                    .execute(c)
            })
            .await;

        match e {
            Ok(x) => JoinServerResult::Success(x as i32),
            Err(x) => {
                dbg!(x);
                JoinServerResult::AlreadyInServer
            }
        }
    }

    pub async fn create_channel_event(
        &self,
        channel_id: i32,
        timestamp: i64,
        event_type: ChannelEventType,
    ) -> Result<usize, diesel::result::Error> {
        let event = event_type.to_event(channel_id, timestamp);
        
        self.run(move |c| {
            diesel::insert_into(db_schema::channel_events::table)
                .values(event)
                .execute(c)
        })
        .await
    }
}
