#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_sync_db_pools;
#[macro_use]
extern crate diesel;

// use chrono::NaiveDateTime;
use diesel::{prelude::*, result::Error};

use rocket::{
    fairing::AdHoc,
    form::Form,
    fs::{relative, FileServer},
    // futures::future::NeverError,
    // State,
    // request::FlashMessage,
    // response::status,
    response::{Flash, Redirect},
    serde::json::{self, Json},
    serde::{Deserialize, Serialize},
    // tokio::time::{sleep, Duration},
    Build,
    Rocket,
};
// use rocket_dyn_templates::Template;

// use parking_lot::Mutex;

// #[database("sqlite_database")]
#[database("diesel")]
pub struct DbConn(diesel::SqliteConnection);

impl DbConn {
    pub async fn get_user_by_id(&self, id: i32) -> Result<User, Error> {
        let form: User = self
            .run(move |conn| users::table.filter(users::id.eq(id)).first(conn))
            .await?;
        Ok(form)
    }

    pub async fn get_user_by_name(&self, name: String) -> Result<User, Error> {
        let form: User = self
            .run(move |conn| users::table.filter(users::username.eq(name)).first(conn))
            .await?;
        Ok(form)
    }

    pub async fn insert_user(&self, user: User) -> Result<usize, Error> {
        let e = self
            .run(move |c| {
                diesel::insert_into(schema::users::table)
                    .values(user)
                    .execute(c)
            })
            .await?;
        Ok(e)
    }

    pub async fn has_user(&self, name: String) -> bool {
        let e = self.get_user_by_name(name).await;
        match e {
            Ok(_) => return true,
            Err(_) => return false,
        }
    }
}

// use rocket::serde::Serialize;
// use diesel::{prelude::*, result::QueryResult};
pub mod schema {
    table! {
        users {
            id -> Nullable<Integer>,
            username -> Text,
            nickname -> Nullable<Text>,
            password -> Text,
            salt -> Text,
            sessions -> Nullable<Blob>,
        }
    }
    table! {
        servers {
            id -> Integer,
            nickname -> Text,
            owner -> Integer,
            channels -> Blob,
            emojis -> Blob,
        }
    }
    table! {
        attachments {
            id -> Integer,
            name -> Text,
            owner -> Integer,
            server -> Integer,
            content -> Blob,
        }
    }
    table! {
        messages {
            id -> Integer,
            sender -> Integer,
            server -> Integer,
            channel -> Integer,
            reply -> Integer,
            text -> Text,
            emoji -> Blob,
            sqltime -> TimestamptzSqlite,
        }
    }
}

// use schema::users;

#[derive(Deserialize, Queryable, Insertable, Debug)]
#[diesel(table_name = schema::users)]
pub struct User {
    id: Option<i32>,
    username: String,
    nickname: Option<String>,
    password: String,
    salt: String,
    sessions: Option<Vec<u8>>,
}

enum InsertError {
    None(usize),
    UsernameTaken,
    DbError(diesel::result::Error),
    InvalidPassword,
}

enum AuthErr {
    None,
    InvalidUsername,
    InvalidPassword,
}

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use schema::users;

impl User {
    async fn insert(new_user: UserAuth, conn: &DbConn) -> InsertError {
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
            salt: salt.to_string(),
            sessions: None,
        };

        let e = conn.insert_user(t).await;
        match e {
            Ok(x) => return InsertError::None(x),
            Err(x) => return InsertError::DbError(x),
        }
    }

    async fn auth(user: UserAuth, conn: DbConn) -> AuthErr {
        let e = conn.get_user_by_name(user.username).await;
        let query;
        match e {
            Err(x) => return AuthErr::InvalidUsername,
            Ok(x) => query = x,
        }

        if query.password == user.password {
            return AuthErr::None;
        } else {
            return AuthErr::InvalidPassword;
        }
    }
}

async fn run_migrations(rocket: Rocket<Build>) -> Rocket<Build> {
    use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

    const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

    DbConn::get_one(&rocket)
        .await
        .expect("database connection")
        .run(|conn| {
            conn.run_pending_migrations(MIGRATIONS)
                .expect("diesel migrations");
        })
        .await;

    rocket
}

#[derive(Debug, FromForm)]
pub struct UserAuth {
    pub username: String,
    pub password: String,
}

#[post("/signup", data = "<new_user>")]
async fn create_user(conn: DbConn, new_user: Form<UserAuth>) -> Flash<Redirect> {
    let user = new_user.into_inner();
    if user.username.is_empty() {
        return Flash::error(Redirect::to("/login"), "username cannot be empty.");
    }

    let err = User::insert(user, &conn).await;

    match err {
        InsertError::None(x) => {
            return Flash::success(Redirect::to("/"), "user successfully added.")
        }
        InsertError::UsernameTaken => {
            return Flash::error(Redirect::to("/login"), "username taken.")
        }
        InsertError::DbError(_x) => {
            return Flash::error(Redirect::to("/login"), "insertion error.")
        }
        InsertError::InvalidPassword => {
            return Flash::error(Redirect::to("/login"), "invalid password.")
        }
    }
}

#[post("/login", data = "<user>")]
async fn auth_user(conn: DbConn, user: Form<UserAuth>) -> Flash<Redirect> {
    let user = user.into_inner();

    if user.username.is_empty() {
        return Flash::error(Redirect::to("/login"), "username cannot be empty.");
    }

    // let err = User::insert(user, &conn).await;
    let err = User::auth(user, conn).await;

    match err {
        AuthErr::None => {
            return Flash::success(Redirect::to("/"), "user successfully authenticated.")
        }
        AuthErr::InvalidUsername => {
            return Flash::error(Redirect::to("/login"), "invalid username.")
        }
        AuthErr::InvalidPassword => {
            return Flash::error(Redirect::to("/login"), "invalid password.")
        }
    }
}

#[launch]
fn rocket() -> _ {
    rocket::build()
        .attach(DbConn::fairing())
        // .attach(Template::fairing())
        .attach(AdHoc::on_ignite("Run Migrations", run_migrations))
        .mount("/", FileServer::from(relative!("static")))
        .mount("/", routes![create_user, auth_user])
    // .mount("/", routes![index])
    // .mount("/todo", routes![new, toggle, delete])
    // .mount("/", routes![inbox, system, test])
}
