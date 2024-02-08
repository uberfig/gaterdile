use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
// use chrono::NaiveDateTime;
use crate::schema::schema::{self, channels, server_members};
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
#[diesel(table_name = schema::users)]
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

        let pass;
        match password_hash {
            Ok(x) => pass = x.to_string(),
            Err(_) => return InsertError::InvalidPassword,
        }

        let t = User {
            id: None,
            username: new_user.username,
            nickname: None,
            password: pass,
        };

        let e = conn.insert_user(t).await;
        match e {
            Ok(x) => return InsertError::Success(x),
            Err(_x) => {
                println!("insert");
                dbg!(_x);
                return InsertError::DbError;
            }
        }
    }

    pub async fn auth(user: UserAuth, conn: &DbConn) -> AuthErr {
        let e = conn.get_user_by_name(user.username).await;
        let query;
        match e {
            Err(_x) => return AuthErr::InvalidUsername,
            Ok(x) => query = x,
        }

        let password_hash = PasswordHash::new(&query.password).unwrap();
        let verified = Argon2::default().verify_password(user.password.as_bytes(), &password_hash);

        match verified {
            Ok(_) => return AuthErr::Success(query.id.unwrap()),
            Err(_) => return AuthErr::InvalidPassword,
        }
    }
}

#[derive(Deserialize, Queryable, Insertable, Debug, Serialize, Clone)]
#[diesel(table_name = schema::messages)]
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
#[diesel(table_name = schema::server_members)]
pub struct ServerMember {
    // pub id: Option<i32>,
    pub server_id: i32,
    pub userid: i32,
    pub nickname: Option<String>,
}

#[derive(Deserialize, Queryable, Insertable, Debug, Serialize, Clone)]
#[diesel(table_name = schema::channels)]
pub struct Channel {
    pub id: Option<i32>,
    pub server: i32,
    pub name: String,
}

#[derive(Deserialize, Queryable, Insertable, Debug, Serialize, Clone)]
#[diesel(table_name = schema::channel_events)]
pub struct ChannelEvent {
    pub id: Option<i32>,
    pub channel_id: i32,
    pub timestamp: i64,
    pub event_type: i32,
    pub message: Option<i32>,
    pub reaction: Option<i32>,
    pub user: Option<i32>,
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
use diesel::{prelude::*, result::Error};
use schema::{
    // messages::{self, channel},
    messages::{self},
    // messages::self,
    // usernames, users,
    users,
};

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
                schema::users::table
                    .filter(schema::users::id.eq(id))
                    .first(conn)
            })
            .await?;
        Ok(user.username)
    }

    pub async fn insert_user(&self, user: User) -> Result<usize, Error> {
        self
            .run(move |c| {
                diesel::insert_into(schema::users::table)
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
        let e = self
            .run(move |c| {
                let a = diesel::insert_into(schema::messages::table)
                    .values(message)
                    .execute(c);
                a
            })
            .await;
        e
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
        self
            .run(move |conn| {
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
        self
            .run(move |conn| {
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
        self
            .run(move |conn| {
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
        self
            .run(move |conn| {
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
        self
            .run(move |conn| {
                channels::dsl::channels
                    .filter(channels::dsl::server.eq(server_id))
                    .load::<Channel>(conn)
            })
            .await
    }

    pub async fn join_server(&self, message: ServerMember) -> JoinServerResult {
        let e = self
            .run(move |c| {
                diesel::insert_into(schema::server_members::table)
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

    // pub async fn create_reaction(&self, server_id: i32, channel_id: i32) {
    //     todo!()
    // }

    // pub async fn remove_reaction(&self, server_id: i32, channel_id: i32) {
    //     todo!()
    // }
}
