use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use rocket_db_pools::Connection;
use sqlx::{Error, PgConnection};

use crate::transmission::{AuthErr, InsertResult, UserAuth};

use super::db::DbConn;

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
