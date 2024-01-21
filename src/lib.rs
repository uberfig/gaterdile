#![recursion_limit = "1024"]
#![type_length_limit = "1024"]

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate rocket_sync_db_pools;

use argon2::{
	password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
	Argon2,
};
use rocket::serde::{Deserialize, Serialize};

pub mod schema {
	table! {
		users {
			id -> Nullable<Integer>,
			username -> Text,
			nickname -> Nullable<Text>,
			password -> Text,
		}
	}
	table! {
		servers {
			id -> Nullable<Integer>,
			nickname -> Text,
			owner -> Integer,
		}
	}
	table! {
		emojis {
			id -> Nullable<Integer>,
			server -> Integer,
			name -> Text,
			attachmentid -> Integer,
		}
	}
	table! {
		channels (id, server) {
			id 		-> Nullable<Integer>,
			server 	-> Integer,
			name 	-> Text,
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
			id 		-> Nullable<Integer>,
			sender	-> Integer,
			server	-> Integer,
			channel -> Integer,
			mention -> Blob,
			reply	-> Nullable<Integer>,
			text	-> Text,
			emoji	-> Nullable<Blob>,
			sqltime	-> Nullable<Timestamp>,
		}
	}
	table! {
		mentions {
			id 		-> Integer, //message id
			userid  -> Nullable<Integer>,
			roleid  -> Nullable<Integer>
		}
	}
	table! {
		usernames (username) {
			userid 	-> Integer,
			username-> Text,
		}
	}
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UserAuth {
	pub username: String,
	pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum InsertError {
	Success(usize),
	UsernameTaken,
	DbError,
	InvalidPassword,
	InvalidUsername,
}

#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Deserialize, Queryable, Insertable, Debug)]
#[diesel(table_name = schema::usernames)]
pub struct UsernameMap {
	pub userid: i32,
	pub username: String,
}

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
			Err(_x) => return InsertError::DbError,
		}
	}

	pub async fn auth(user: UserAuth, conn: &DbConn) -> AuthErr {
		let e = conn.get_user_by_name(user.username).await;
		let query;
		match e {
			Err(x) => return AuthErr::InvalidUsername,
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

#[derive(Deserialize, Queryable, Insertable, Debug, Serialize)]
#[diesel(table_name = schema::messages)]
pub struct Message {
	id: Option<i32>,
	sender: i32,
	server: i32,
	channel: i32,
	reply: Option<i32>,
	emoji: Option<Vec<u8>>,
	sqltime: Option<chrono::NaiveDateTime>,
}


#[database("diesel")]
pub struct DbConn(diesel::SqliteConnection);
use schema::{usernames, users};
use diesel::{prelude::*, result::Error};

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

	pub async fn get_user_id(&self, name: String) -> Result<UsernameMap, Error> {
		let form = self
			.run(move |conn| schema::usernames::table.filter(schema::usernames::username.eq(name)).first(conn))
			.await?;
		Ok(form)
		// return form;
	}

	pub async fn insert_user(&self, user: User) -> Result<usize, Error> {
		let username = user.username.clone();

		let e = self
			.run(move |c| {
				let a = diesel::insert_into(schema::users::table)
					.values(user)
					.execute(c);
				a
			})
			.await?;
		// let a = Ok(e);

		let uname_map = UsernameMap {userid: e as i32, username: username};

		let err2 = self
			.run(move |d| {
				let a = diesel::insert_into(schema::usernames::table)
					.values(uname_map)
					.execute(d);
				a
			})
			.await?;


		return Ok(e);
	}

	pub async fn has_user(&self, name: String) -> bool {
		let e = self.get_user_id(name).await;
		match e {
			Ok(_) => return true,
			Err(_) => return false,
		}
	}
}
