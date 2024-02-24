//gaterdile
#![recursion_limit = "4096"]
#![type_length_limit = "4096"]
#[macro_use]
extern crate rocket;

use std::time::Duration;

use gaterdile::handlers::{
    fetch_new_events, handle_auth, handle_create_user, handle_get_channel, handle_get_prior,
    handle_get_server, handle_join_server, handle_send_message, ConnectionProps,
};
use gaterdile::transmission::{Transmission, TransmissionType};

use rocket::tokio;
use rocket::tokio::time::interval;

use rocket::{
    fairing::AdHoc,
    fs::{relative, FileServer},
    Build, Rocket,
};

use gaterdile::db::DbConn;
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
        TransmissionType::GetRoom(server_id, channel_id) => {
            handle_get_channel(server_id, channel_id, props, conn, stream).await;
        }
        TransmissionType::GetServer(server_id) => {
            handle_get_server(server_id, conn, stream).await;
        }
        TransmissionType::CreateUser(x) => {
            handle_create_user(x, props, conn, stream).await;
        }
        TransmissionType::GetUserServers => {
            todo!()
        }
        TransmissionType::JoinServer(server_id) => {
            handle_join_server(server_id, props.uid, conn, stream).await;
        }
        TransmissionType::GetPriorMessages(since) => {
            handle_get_prior(props.listening_channel.unwrap_or(-1), conn, stream, since).await
        }
        TransmissionType::GetEmoji(_) => todo!(),
        TransmissionType::GetAttachment(_) => todo!(),
        TransmissionType::CreateCommunity(_) => todo!(),
        TransmissionType::CreateRoom(_, _) => todo!(),

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
        | TransmissionType::UserEvent(_) => {
            let _ = Transmission::invalid().send(stream).await;
        }
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
