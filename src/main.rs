//gaterdile
#![recursion_limit = "4096"]
#![type_length_limit = "4096"]
#[macro_use]
extern crate rocket;

use std::time::Duration;

use gaterdile::db_types::ChannelEvent;
use gaterdile::handlers::{handle_get_channel, handle_get_prior, handle_get_server, handle_join_server, ConnectionProps};
use gaterdile::transmission::{
    AuthErr, InsertError, Transmission, NewTransmissionMessage, TransmissionType, UserAuth
};

use rocket::futures;
use rocket::tokio::time::interval;
use rocket::tokio;

use rocket::{
    fairing::AdHoc,
    fs::{relative, FileServer},
    Build,
    Rocket,
};

use gaterdile::db::{DbConn, User};
use rocket_ws as ws;

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

async fn create_user(conn: &DbConn, user: UserAuth) -> InsertError {
    if user.username.is_empty() {
        return InsertError::InvalidUsername;
    }

    User::insert(user, conn).await
}

async fn auth_user(conn: &DbConn, user: UserAuth) -> AuthErr {
    if user.username.is_empty() {
        return AuthErr::InvalidUsername;
    }
    User::auth(user, conn).await
}

async fn handle_send_message(
    t_msg: NewTransmissionMessage,
    props: &mut ConnectionProps,
    conn: &DbConn,
    stream: &mut ws::stream::DuplexStream,
) {
    if t_msg.text.trim().is_empty() {
        let _ = TransmissionType::InvalidTransmission
            .wrap_into_transmission()
            .send(stream)
            .await;
        return;
    }
    let message = t_msg.to_message(props.uid);
    let _x = conn.send_message(message).await;
    // dbg!(_x);

    props.listening_server = Some(props.listening_server.unwrap_or(t_msg.server));
    props.listening_channel = Some(props.listening_server.unwrap_or(t_msg.channel));
    fetch_new_events(props, conn, stream).await;
}

async fn handle_auth(
    user: UserAuth,
    props: &mut ConnectionProps,
    conn: &DbConn,
    stream: &mut ws::stream::DuplexStream,
) {
    let auth = auth_user(conn, user).await;
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
        TransmissionType::Reaction(_) => {
            todo!()
        }
        TransmissionType::Auth(user) => {
            handle_auth(user, props, conn, stream).await;
        }
        TransmissionType::GetChannel(server_id, channel_id) => {
            handle_get_channel(server_id, channel_id, props, conn, stream).await;
        }
        TransmissionType::GetServer(server_id) => {
            handle_get_server(server_id, conn, stream).await;
        }
        TransmissionType::CreateUser(x) => {
            let err = create_user(conn, x).await;
            match err {
                InsertError::Success(x) => {
                    props.authenticated = true;
                    props.uid = x.try_into().unwrap();
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
        TransmissionType::GetUserServers => {
            todo!()
        }
        TransmissionType::JoinServer(server_id) => {
            handle_join_server(server_id, props.uid, conn, stream).await;
        }
        TransmissionType::GetPriorMessages(since) => {
            handle_get_prior(
                props.listening_server.unwrap_or(-1),
                props.listening_channel.unwrap_or(-1),
                conn,
                stream,
                since,
            )
            .await
        }
        TransmissionType::GetEmoji(_) => todo!(),
        TransmissionType::GetAttachment(_) => todo!(),

        //-----------------------------invalid types from client------------------------------------
        TransmissionType::InvalidTransmission
        | TransmissionType::RequestAuth
        | TransmissionType::AuthResult(_)
        | TransmissionType::CreateUserResult(_)
        | TransmissionType::ServerInfo(_)
        | TransmissionType::UserServers(_)
        | TransmissionType::JoinServerResult(_)
        | TransmissionType::PriorMessages(_)
        | TransmissionType::NoMorePrior 
        | TransmissionType::ChannelEvent(_)
        | TransmissionType::ServerEvent(_)
        | TransmissionType::UserEvent(_)=> {
            let _ = Transmission::invalid().send(stream).await;
        }
        
    }
}

async fn fetch_new_events(
    props: &mut ConnectionProps,
    conn: &DbConn,
    stream: &mut ws::stream::DuplexStream,
) {
    if props.listening_channel.is_none() || props.listening_server.is_none() {
        return;
    }

    if props.last_sent_timestamp.is_none() {
        handle_get_channel(
            props.listening_server.unwrap(),
            props.listening_channel.unwrap(),
            props,
            conn,
            stream,
        )
        .await;
        return;
    }

    let x = props.last_sent_timestamp.unwrap();

    let since = conn
        .get_events_since_timestamp_and_id(
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
                    // let messages = since.into_iter().filter(ChannelEvent::is_message).map(|y| y.get_message(conn));
                    // let messages = futures::future::join_all(messages).await;
                    let messages = since.into_iter().filter(ChannelEvent::is_message).map(|y| y.get_concrete_unwrap(conn));
                    let messages = futures::future::join_all(messages).await;
                    let _ = TransmissionType::ChannelEvent(messages)
                        .wrap_into_transmission()
                        .send(stream)
                        .await;
                } 
                None => {
                    // println!("no new messages")
                }
            }
        }
        Err(e) => println!("no new messages or db errr {}", e),
    }
}

//with thanks to this issue I found online: https://stackoverflow.com/questions/77780189/how-to-detect-rust-rocket-ws-client-disconnected-from-websocket
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

                let _ = TransmissionType::JoinServer(0).wrap_into_transmission().send(&mut stream).await;
				loop {
					tokio::select! {
						_ = interval.tick() => {
							// Send message every 10 seconds
							if props.authenticated {
								fetch_new_events(&mut props, &conn, &mut stream).await;
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
        .mount("/", FileServer::from(relative!("client/static")))
        .mount("/", routes![message_channel])
}
