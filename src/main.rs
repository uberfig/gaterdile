//gatordile
#![recursion_limit = "1024"]
#![type_length_limit = "1024"]
#[macro_use]
extern crate rocket;


use std::time::Duration;

//
// use diesel::date_time_expr;
use chat::schema;
use chat::schema::*;
use rocket::tokio;
use rocket::tokio::time::interval;
use schema::{usernames, users};

use chrono::Utc;
use chrono::{DateTime, NaiveDateTime};
use diesel::deserialize;
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
	time::{OffsetDateTime, PrimitiveDateTime},
	// tokio::time::{sleep, Duration},
	Build,
	Rocket,
};

use argon2::{
	password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
	Argon2,
};

use rocket_ws as ws;
use chat::{
	DbConn,
	InsertError,
	UserAuth,
	AuthErr,
	User,
	Message,
};

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



// #[post("/signup", data = "<new_user>")]
async fn create_user(conn: &DbConn, user: UserAuth) -> InsertError {
	// let user = new_user.into_inner();
	if user.username.is_empty() {
		return InsertError::InvalidUsername;
	}

	let err = User::insert(user, conn).await;

	return err;
}

// #[post("/login", data = "<user>")]
async fn auth_user(conn: &DbConn, user: UserAuth) -> AuthErr {
	// let user = user.into_inner();

	if user.username.is_empty() {
		return AuthErr::InvalidUsername;
	}
	return User::auth(user, conn).await;
}

#[derive(Debug, Deserialize, Serialize)]
struct NewMessage {
	text: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct React {
	reaction: String,
	message_id: i32,
}

#[derive(Debug, Deserialize, Serialize)]
struct Channel {
	id: i32,
	name: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct ServerInfo {
	id: i32,
	chanels: Vec<Channel>
}

#[derive(Debug, Deserialize, Serialize)]
struct ChannelInfo {
	messages: Vec<Message>,
}

#[derive(Debug, Deserialize, Serialize)]
enum TransmissionType {
	NewMessage(Message),
	SendMessage(Message),
	Reaction(React),
	RequestAuth, //from server
	// Salt(String),        //provides salt
	Auth(UserAuth),
	AuthResult(AuthErr),
	GetServer(i32),	//requests to set the current server and get server info
	GetChannel(i32), //requests the channel from the current selected server
	InvalidTransmission,
	CreateUser(UserAuth),
	CreateUserResult(InsertError),
}

impl std::fmt::Display for TransmissionType {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
    TransmissionType::NewMessage(_) => write!(f, "NewMessage"),
    TransmissionType::SendMessage(_) => write!(f, "SendMessage"),
    TransmissionType::Reaction(_) => write!(f, "Reaction"),
    TransmissionType::RequestAuth => write!(f, "RequestAuth"),
    TransmissionType::Auth(_) => write!(f, "Auth"),
    TransmissionType::AuthResult(_) => write!(f, "AuthResult"),
    TransmissionType::GetServer(_) => write!(f, "GetServer"),
    TransmissionType::GetChannel(_) => write!(f, "GetChannel"),
    TransmissionType::InvalidTransmission => write!(f, "InvalidTransmission"),
    TransmissionType::CreateUser(_) => write!(f, "CreateUser"),
    TransmissionType::CreateUserResult(_) => write!(f, "CreateUserResult"),
		}
	}
}

#[derive(Debug, Deserialize, Serialize)]
struct Transmission {
	data: TransmissionType,
	transmission_type: String,
}

impl Transmission {
	pub fn stringify(&self) -> String {
		serde_json::to_string(&self).unwrap()
	}
	pub fn parse(val: &str) -> Result<Self, ()> {
		let a = serde_json::from_str::<Transmission>(val);
		match a {
			Ok(x) => return Result::Ok(x),
			Err(x) => return Result::Err(()),
		}
	}
}

impl TransmissionType {
	pub fn stringify(&self) -> String {
		serde_json::to_string(&self).unwrap()
	}
	pub fn parse(val: &str) -> Result<Self, ()> {
		let a = serde_json::from_str::<TransmissionType>(val);
		match a {
			Ok(x) => return Result::Ok(x),
			Err(x) => return Result::Err(()),
		}
	}
}

struct ConnectionProps {
	uid: i32,
	authenticated: bool,
	listening_server: Option<i32>,
	listening_channel: Option<i32>,
}

async fn handle_transmission(transmission: TransmissionType, props: &mut ConnectionProps, conn: &DbConn, stream: &mut ws::stream::DuplexStream) {
	use rocket::futures::SinkExt;
	println!("handleing: {}", transmission);

	match transmission {
		TransmissionType::NewMessage(x) => todo!(),
		TransmissionType::SendMessage(x) => todo!(),
		TransmissionType::Reaction(_) => todo!(),
		TransmissionType::RequestAuth => todo!(),
		TransmissionType::Auth(user) => {
			let auth = auth_user(&conn, user).await;
			match auth {
				AuthErr::Success(x) => {
					props.authenticated = true;
					props.uid = x;
	
					let _ = stream
								.send(rocket_ws::Message::Text(
									TransmissionType::AuthResult(auth).stringify(),
								))
								.await;
				},
				_ => {
					props.authenticated = false;
					props.uid = -1;
					let _ = stream
								.send(rocket_ws::Message::Text(
									TransmissionType::AuthResult(auth).stringify(),
								))
								.await;
				}
			}
		},
		TransmissionType::AuthResult(_) => {
			let _ = stream
						.send(rocket_ws::Message::Text(
							TransmissionType::InvalidTransmission.stringify(),
						))
						.await;
		},
		TransmissionType::GetChannel(x) => todo!(),
		TransmissionType::GetServer(x) => todo!(),
		TransmissionType::InvalidTransmission => {
			let _ = stream
						.send(rocket_ws::Message::Text(
							TransmissionType::InvalidTransmission.stringify(),
						))
						.await;
		},
    TransmissionType::CreateUser(x) => {
		let err = create_user(conn, x).await;
		match err {
			InsertError::Success(x) => {
				props.authenticated = true;
				props.uid = x as i32;
			},
			_ => {},
		}
		let _ = stream
						.send(rocket_ws::Message::Text(
							TransmissionType::CreateUserResult(err).stringify(),
						))
						.await;
	},
    TransmissionType::CreateUserResult(x) => {
		let _ = stream
						.send(rocket_ws::Message::Text(
							TransmissionType::InvalidTransmission.stringify(),
						))
						.await;
	},
    
	}
}

//https://stackoverflow.com/questions/77780189/how-to-detect-rust-rocket-ws-client-disconnected-from-websocket
#[get("/ws")]
pub fn message_channel(ws: ws::WebSocket, conn: DbConn) -> ws::Channel<'static> {
	use rocket::futures::{SinkExt, StreamExt};

	ws.channel(move |mut stream: ws::stream::DuplexStream| {
		Box::pin(async move {
			let mut interval = interval(Duration::from_secs(6));
			let mut props = ConnectionProps {uid: -1, authenticated:false, listening_server: None, listening_channel: None };

			tokio::spawn(async move {
				let _ = stream
					.send(rocket_ws::Message::Text(
						TransmissionType::RequestAuth.stringify(),
					))
					.await;


					let _ = stream
					.send(rocket_ws::Message::Text(
						Transmission { data: TransmissionType::RequestAuth, transmission_type: TransmissionType::RequestAuth.to_string() }
						.stringify(),
					))
					.await;

					let react = TransmissionType::Reaction(React { reaction: "🙂".to_string(), message_id: 0});

					let name = react.to_string();
					let _ = stream
					.send(rocket_ws::Message::Text(
						Transmission { data: react, transmission_type: name }
						.stringify(),
					))
					.await;

					let auth = TransmissionType::Auth(UserAuth { username: "ivy".to_string(), password: "123".to_string() });

					let name = auth.to_string();
					let _ = stream
					.send(rocket_ws::Message::Text(
						Transmission { data: auth, transmission_type: name }
						.stringify(),
					))
					.await;


				// let _ = stream.send(ws::Message::Text(Transmission::Reaction(React { reaction: "🙂".to_string(), message_id: 0 }).stringify())).await;

				loop {
					tokio::select! {
						_ = interval.tick() => {
							// Send message every 10 seconds
							// let reading = get_latest_readings().await.unwrap();
							// let reading = get_latest_readings().await.unwrap();
							// let _ = stream.send(ws::Message::Text(serde_json::to_string(reading).unwrap())).await;
							
							// println!("Sent message");

							// let _ = stream.send(ws::Message::Text("hello".to_string())).await;
							// let _ = stream.send(ws::Message::Text(Transmission::Reaction(React { reaction: "🙂".to_string(), message_id: 0 }).stringify())).await;
						}
						Some(Ok(message)) = stream.next() => {
							match message {
								ws::Message::Text(text) => {
									// Handle Text message
									println!("Received Text message: {}", text);
									let data = Transmission::parse(&text).unwrap_or(Transmission { data: TransmissionType::InvalidTransmission, transmission_type: "".to_string() });

									// let parsed = TransmissionType::parse(&text).unwrap_or(TransmissionType::InvalidTransmission);
									handle_transmission(data.data, &mut props, &conn, &mut stream).await;
								}
								ws::Message::Binary(data) => {
									// Handle Binary message
									println!("Received Binary message: {:?}", data);
								}
								ws::Message::Close(close_frame) => {
									// Handle Close message
									println!("Received Close message: {:?}", close_frame);
									let close_frame = ws::frame::CloseFrame {
										code: ws::frame::CloseCode::Normal,
										reason: "Client disconected".to_string().into(),
									};
									let _ = stream.close(Some(close_frame)).await;
									break;
								}
								ws::Message::Ping(ping_data) => {
									// Handle Ping message
									println!("Received Ping message: {:?}", ping_data);
								}
								ws::Message::Pong(pong_data) => {
									// Handle Pong message
									println!("Received Pong message: {:?}", pong_data);
								}
								_ => {
									println!("Received other message: {:?}", message);
								}
							}
						}
						else => {
							println!("Connection closed");
							let close_frame = ws::frame::CloseFrame {
								code: ws::frame::CloseCode::Normal,
								reason: "Client disconected".to_string().into(),
							};
							let _ = stream.close(Some(close_frame)).await;
							// The connection is closed by the client
							break;
						}
					}
				}
			});

			tokio::signal::ctrl_c().await.unwrap();
			Ok(())
		})
	})
}

#[launch]
fn rocket() -> _ {
	rocket::build()
		.attach(DbConn::fairing())
		// .attach(Template::fairing())
		.attach(AdHoc::on_ignite("Run Migrations", run_migrations))
		.mount("/", FileServer::from(relative!("static")))
		.mount("/", routes![message_channel])
}
