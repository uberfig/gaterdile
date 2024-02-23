use crate::{
    db_types::{Channel, ChannelEvent, ChannelEventType, Message, ServerMember},
    schema::db_schema::{self, channel_events, channels, server_members},
    transmission::{AuthErr, InsertError, JoinServerResult, UserAuth},
};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rocket::serde::Deserialize;

#[derive(Deserialize, Queryable, Insertable, Debug)]
#[diesel(table_name = db_schema::users)]
pub struct User {
    pub id: Option<i64>,
    pub username: String,
    pub nickname: Option<String>,
    password: String,
}

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

#[database("diesel")]
pub struct DbConn(diesel::PgConnection);
use db_schema::{
    // messages::{self, channel},
    messages::{self},
    // messages::self,
    // usernames, users,
    users,
};
use diesel::{prelude::*, result::Error, sql_types::Integer};

#[derive(QueryableByName)]
pub struct InsertedRowId {
    #[diesel(sql_type = Integer)]
    pub id: i32,
}

impl DbConn {
    pub async fn get_user_by_id(&self, id: i64) -> Result<User, Error> {
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

    pub async fn get_user_name(&self, id: i64) -> Result<String, Error> {
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

    pub async fn get_msg_by_id(&self, id: i64) -> Result<Message, Error> {
        let message: Message = self
            .run(move |conn| messages::table.filter(messages::id.eq(id)).first(conn))
            .await?;
        Ok(message)
    }

    pub async fn send_message(&self, message: Message) -> Result<Message, diesel::result::Error> {
        let timestamp = message.timestamp;
        let channel_id = message.channel;
        let server_id = message.server;

        let err_t = self
            .run(move |c| {
                c.transaction(|c| {
                    let _a = diesel::insert_into(db_schema::messages::table)
                        .values(message)
                        .get_result::<Message>(c);
                    diesel::result::QueryResult::Ok(_a)
                })
            })
            .await?;

        match &err_t {
            Ok(x) => {
                println!("inserting {}", x.id.unwrap());
                let y = self
                    .create_channel_event(
                        channel_id,
                        server_id,
                        timestamp,
                        ChannelEventType::NewMessage(x.id.unwrap()),
                    )
                    .await;
                dbg!(&y);
                let _restut = y.expect("unable to insert message event for new message");
            }
            Err(_) => todo!(),
        }

        err_t

        // return Ok(1);
    }

    pub async fn get_server_members(
        &self,
        server_id: i64,
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
        uid: i64,
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
        server_id: i64,
    ) -> Result<Vec<Channel>, diesel::result::Error> {
        self.run(move |conn| {
            channels::dsl::channels
                .filter(channels::dsl::server.eq(server_id))
                .load::<Channel>(conn)
        })
        .await
    }

    pub async fn join_server(
        &self,
        server_id: i64,
        userid: i64,
        nickname: Option<String>,
    ) -> JoinServerResult {
        let message: ServerMember = ServerMember {
            server_id,
            userid,
            nickname,
        };
        let e = self
            .run(move |c| {
                diesel::insert_into(db_schema::server_members::table)
                    .values(message)
                    .execute(c)
            })
            .await;

        match e {
            Ok(x) => JoinServerResult::Success(x.try_into().unwrap()),
            Err(x) => {
                dbg!(x);
                JoinServerResult::AlreadyInServer
            }
        }
    }

    pub async fn create_channel_event(
        &self,
        channel_id: i64,
        server_id: i64,
        timestamp: i64,
        event_type: ChannelEventType,
    ) -> Result<usize, diesel::result::Error> {
        let event = event_type.to_event(channel_id, server_id, timestamp);

        self.run(move |c| {
            diesel::insert_into(db_schema::channel_events::table)
                .values(event)
                .execute(c)
        })
        .await
    }

    pub async fn get_events_since_timestamp_and_id(
        &self,
        channel_id: i64,
        since: i64,
        id: i64,
        amount: i64,
    ) -> Result<Vec<ChannelEvent>, diesel::result::Error> {
        let mut a = self
            .run(move |conn| {
                channel_events::dsl::channel_events
                    .filter(channel_events::dsl::channel_id.eq(channel_id))
                    .filter(channel_events::dsl::timestamp.ge(since))
                    .order(channel_events::dsl::timestamp.desc())
                    .limit(amount)
                    .filter(channel_events::dsl::id.ne(id))
                    .load::<ChannelEvent>(conn)
            })
            .await;

        match &mut a {
            Ok(x) => {
                x.sort_unstable_by_key(|y| y.timestamp);
            }
            Err(_) => {}
        }

        a
    }

    pub async fn get_events_prior(
        &self,
        // server_id: i32,
        channel_id: i64,
        prior_to: i64,
        last_msg: i64,
        amount: i64,
    ) -> Result<Vec<ChannelEvent>, Error> {
        let mut val = self
            .run(move |conn| {
                channel_events::dsl::channel_events
                    // .filter(messages::dsl::server.eq(server_id))
                    .filter(channel_events::dsl::channel_id.eq(channel_id))
                    .filter(channel_events::dsl::timestamp.le(prior_to))
                    .filter(channel_events::dsl::id.ne(last_msg))
                    .order(channel_events::dsl::timestamp.asc())
                    .limit(amount)
                    // .order(messages::dsl::id.desc())
                    // .order(messages::dsl::timestamp.asc())
                    .load::<ChannelEvent>(conn)
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

    pub async fn get_channel_events(
        &self,
        // server_id: i32,
        channel_id: i64,
        amount: i64,
    ) -> Result<Vec<ChannelEvent>, Error> {
        let mut val = self
            .run(move |conn| {
                channel_events::dsl::channel_events
                    // .filter(messages::dsl::server.eq(server_id))
                    .filter(channel_events::dsl::channel_id.eq(channel_id))
                    .order(channel_events::dsl::timestamp.desc())
                    .limit(amount)
                    // .order(messages::dsl::id.desc())
                    // .order(messages::dsl::timestamp.asc())
                    .load::<ChannelEvent>(conn)
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
}
