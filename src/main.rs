//gatordile
#![recursion_limit = "2048"]
#![type_length_limit = "2048"]
#[macro_use]
extern crate rocket;

use std::time::Duration;

//
// use diesel::date_time_expr;
use rocket::futures::SinkExt;
use rocket::tokio;
use rocket::tokio::time::interval;

use rocket::{
    fairing::AdHoc,
    fs::{relative, FileServer},
    // futures::future::NeverError,
    // State,
    // request::FlashMessage,
    // response::status,
    // response::{Flash, Redirect},
    // serde::json::{self, Json},
    serde::{Deserialize, Serialize},
    // time::{OffsetDateTime, PrimitiveDateTime},
    // tokio::time::{sleep, Duration},
    Build,
    Rocket,
};

// use argon2::{
//     password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
//     Argon2,
// };

use gaterdile::db::{AuthErr, DbConn, InsertError, Message, User, UserAuth};
use rocket_ws as ws;
// use serde::ser;

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

#[derive(Debug, Deserialize, Serialize, Clone)]
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
    chanels: Vec<Channel>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
struct ChannelInfo {
    messages: Vec<Message>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TransmissionMessage {
    server: i32,
    channel: i32,
    reply: Option<i32>,
    text: String,
}

impl TransmissionMessage {
    fn to_message(&self, uid: i32) -> Message {
        use std::time::SystemTime;
        Message {
            id: None,
            sender: uid,
            server: self.server,
            channel: self.channel,
            reply: self.reply,
            text: self.text.clone(),
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
enum TransmissionType {
    SendMessage(TransmissionMessage),
    Reaction(React),
    Auth(UserAuth),
    GetServer(i32),       //requests to set the current server and get server info
    GetChannel(i32, i32), //server, channel
    CreateUser(UserAuth),
    //from server only:
    InvalidTransmission,
    NewMessages(Vec<Message>),
    RequestAuth,
    AuthResult(AuthErr),
    CreateUserResult(InsertError),
}

impl std::fmt::Display for TransmissionType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TransmissionType::NewMessages(_) => write!(f, "NewMessages"),
            TransmissionType::SendMessage(_) => write!(f, "SendMessage"),
            TransmissionType::Reaction(_) => write!(f, "Reaction"),
            TransmissionType::RequestAuth => write!(f, "RequestAuth"),
            TransmissionType::Auth(_) => write!(f, "Auth"),
            TransmissionType::AuthResult(_) => write!(f, "AuthResult"),
            TransmissionType::GetServer(_) => write!(f, "GetServer"),
            TransmissionType::GetChannel(..) => write!(f, "GetChannel"),
            TransmissionType::CreateUser(_) => write!(f, "CreateUser"),
            TransmissionType::InvalidTransmission => write!(f, "InvalidTransmission"),
            TransmissionType::CreateUserResult(_) => write!(f, "CreateUserResult"),
        }
    }
}

impl TransmissionType {
    pub fn wrap_into_transmission(self) -> Transmission {
        let name = self.to_string();
        Transmission {
            data: self,
            transmission_type: name,
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
            Err(_x) => return Result::Err(()),
        }
    }
    pub fn invalid() -> Transmission {
        Transmission {
            data: TransmissionType::InvalidTransmission,
            transmission_type: TransmissionType::InvalidTransmission.to_string(),
        }
    }
    pub async fn send(
        &self,
        stream: &mut ws::stream::DuplexStream,
    ) -> Result<(), ws::result::Error> {
        let _a = stream
            .send(rocket_ws::Message::Text(self.stringify()))
            .await;
        _a
    }
}

async fn handle_send_message(
    t_msg: TransmissionMessage,
    props: &mut ConnectionProps,
    conn: &DbConn,
    stream: &mut ws::stream::DuplexStream,
) {
    if t_msg.text.trim().is_empty() {
        let _ = TransmissionType::InvalidTransmission.wrap_into_transmission().send(stream).await;
        return;
    }
    let message = t_msg.to_message(props.uid);
    let _ = conn.send_message(message).await;

    props.listening_server = Some(props.listening_server.unwrap_or(t_msg.server));
    props.listening_channel = Some(props.listening_server.unwrap_or(t_msg.channel));
    fetch_new_messages(props, conn, stream).await;
}

async fn handle_auth(
    user: UserAuth,
    props: &mut ConnectionProps,
    conn: &DbConn,
    stream: &mut ws::stream::DuplexStream,
) {
    let auth = auth_user(&conn, user).await;
    match auth {
        AuthErr::Success(x) => {
            props.authenticated = true;
            props.uid = x;
        }
        _ => {
            props.authenticated = false;
            props.uid = -1;
        }
    }
    let a_result = TransmissionType::AuthResult(auth);
    let name = a_result.to_string();
    let _ = Transmission {
        data: a_result,
        transmission_type: name,
    }
    .send(stream)
    .await;
}

async fn handle_get_channel(
    server_id: i32,
    channel_id: i32,
    props: &mut ConnectionProps,
    conn: &DbConn,
    stream: &mut ws::stream::DuplexStream,
) {
    let a = conn.get_channel_messages(server_id, channel_id, 10).await;
    match a {
        Ok(x) => {
            props.listening_channel = Some(channel_id);
            props.listening_server = Some(server_id);
            let newlast = x.get(x.len().wrapping_sub(1));
            match newlast {
                Some(y) => {
                    props.last_sent_timestamp = Some(y.timestamp);
                    props.last_sent_id = Some(y.id.unwrap());

                    println!("newlast id: ");
                    dbg!(y.id);
                }
                None => {
                    println!("no messages")
                }
            }

            let _ = TransmissionType::NewMessages(x)
                .wrap_into_transmission()
                .send(stream)
                .await;
        }
        Err(_) => {}
    }
}

#[derive(Debug)]
struct ConnectionProps {
    uid: i32,
    authenticated: bool,
    listening_server: Option<i32>,
    listening_channel: Option<i32>,
    last_sent_timestamp: Option<i64>,
    last_sent_id: Option<i32>,
}

async fn handle_transmission(
    transmission: TransmissionType,
    props: &mut ConnectionProps,
    conn: &DbConn,
    stream: &mut ws::stream::DuplexStream,
) {
    // use rocket::futures::SinkExt;
    println!("handleing: {}", transmission);

    match transmission {
        TransmissionType::SendMessage(t_msg) => {
            handle_send_message(t_msg, props, conn, stream).await;
        }
        TransmissionType::Reaction(_) => todo!(),

        TransmissionType::Auth(user) => {
            handle_auth(user, props, conn, stream).await;
        }

        TransmissionType::GetChannel(server_id, channel_id) => {
            handle_get_channel(server_id, channel_id, props, conn, stream).await;
        }
        TransmissionType::GetServer(_x) => todo!(),

        TransmissionType::CreateUser(x) => {
            let err = create_user(conn, x).await;
            match err {
                InsertError::Success(x) => {
                    props.authenticated = true;
                    props.uid = x as i32;
                }
                _ => {
                    props.authenticated = false;
                    props.uid = -1;
                }
            }
            let _ = TransmissionType::CreateUserResult(err)
                .wrap_into_transmission()
                .send(stream)
                .await;
        }

        //invalid types from client
        TransmissionType::InvalidTransmission => {
            let _ = Transmission::invalid().send(stream).await;
        }
        TransmissionType::NewMessages(_) => {
            let _ = Transmission::invalid().send(stream).await;
        }
        TransmissionType::RequestAuth => {
            let _ = Transmission::invalid().send(stream).await;
        }
        TransmissionType::AuthResult(_) => {
            let _ = Transmission::invalid().send(stream).await;
        }
        TransmissionType::CreateUserResult(_) => {
            let _ = Transmission::invalid().send(stream).await;
        }
    }
}

async fn fetch_new_messages(
    props: &mut ConnectionProps,
    conn: &DbConn,
    stream: &mut ws::stream::DuplexStream,
) {
    if matches!(props.listening_channel, None) || matches!(props.listening_server, None) {
        return;
    }

    if matches!(props.last_sent_timestamp, None) {
        handle_get_channel(props.listening_server.unwrap(), props.listening_channel.unwrap(), props, conn, stream).await;
        return;
    }

    match props.last_sent_timestamp {
        Some(x) => {
            let since = conn
                .get_messages_since_timestamp_and_id(
                    props.listening_server.unwrap(),
                    props.listening_channel.unwrap(),
                    x,
                    props.last_sent_id.unwrap(),
                    10,
                )
                .await;
            match since {
                Ok(since) => {
                    let newlast = since.get(since.len().wrapping_sub(1));
                    match newlast {
                        Some(y) => {
                            if y.id == props.last_sent_id {
                                return;
                            }
                            println!("newlast id: ");
                            dbg!(y.id);

                            props.last_sent_timestamp = Some(y.timestamp);
                            props.last_sent_id = Some(y.id.unwrap());
                            let _ = TransmissionType::NewMessages(since)
                                .wrap_into_transmission()
                                .send(stream)
                                .await;
                        }
                        None => {
                            println!("no new messages")
                        }
                    }
                }
                Err(e) => println!("no new messages or db errr {}", e),
            }
        }
        None => println!("last sent not set!"),
    }
}

//https://stackoverflow.com/questions/77780189/how-to-detect-rust-rocket-ws-client-disconnected-from-websocket
#[get("/ws")]
pub fn message_channel(ws: ws::WebSocket, conn: DbConn) -> ws::Channel<'static> {
    // use rocket::futures::{SinkExt, StreamExt};
    use rocket::futures::StreamExt;

    ws.channel(move |mut stream: ws::stream::DuplexStream| {
		Box::pin(async move {
			let mut interval = interval(Duration::from_secs(1));
			let mut props = ConnectionProps {uid: -1, authenticated:false, listening_server:None, listening_channel:None, last_sent_timestamp: None, last_sent_id: None };

			tokio::spawn(async move {
				let _ = Transmission { data: TransmissionType::RequestAuth, transmission_type: TransmissionType::RequestAuth.to_string() }.send(&mut stream).await;

                let _ = TransmissionType::GetChannel(1,2).wrap_into_transmission().send(&mut stream).await;
				loop {
					tokio::select! {
						_ = interval.tick() => {
							// Send message every 10 seconds
							if props.authenticated {
								fetch_new_messages(&mut props, &conn, &mut stream).await;
							}
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
