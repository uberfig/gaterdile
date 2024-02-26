use crate::{
    db::{DbConn, User},
    db_event_types::RoomEvent,
    db_types::{Message, Room, ServerMember},
    transmission::{
        AuthErr, InsertError, NewTransmissionMessage, ServerInfoData, Transmission,
        TransmissionType, UserAuth,
    },
};
use rocket::futures;
use rocket::tokio::join;
use rocket_ws as ws;

#[derive(Debug)]
pub struct ConnectionProps {
    pub uid: i64,
    pub authenticated: bool,
    pub listening_server: Option<i64>,
    pub listening_channel: Option<i64>,
    pub last_sent_timestamp: Option<i64>,
    pub last_sent_id: Option<i64>,
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

pub async fn fetch_new_events(
    props: &mut ConnectionProps,
    conn: &DbConn,
    stream: &mut ws::stream::DuplexStream,
) {
    //----------------get user events----------------



    //--------------get community events-------------

    //----------------get room events----------------

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
        .get_room_events_since_timestamp_and_id(
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

                    let messages = since
                        .into_iter()
                        .filter(RoomEvent::is_message)
                        .map(|y| y.get_concrete_unwrap(conn));
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

pub async fn handle_send_message(
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

    props.listening_server = Some(props.listening_server.unwrap_or(t_msg.server));
    props.listening_channel = Some(props.listening_server.unwrap_or(t_msg.channel));

    let message = Message::from_newmsg(t_msg, props.uid);
    let _x = conn.send_message(message).await;

    fetch_new_events(props, conn, stream).await;
}

pub async fn handle_auth(
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

pub async fn handle_create_user(
    x: UserAuth,
    props: &mut ConnectionProps,
    conn: &DbConn,
    stream: &mut ws::stream::DuplexStream,
) {
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

pub async fn handle_get_channel(
    server_id: i64,
    channel_id: i64,
    props: &mut ConnectionProps,
    conn: &DbConn,
    stream: &mut ws::stream::DuplexStream,
) {
    let a = conn.get_channel_events(channel_id, 40).await;

    if let Ok(x) = a {
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
                // println!("no messages")
            }
        }
        let messages = x
            .into_iter()
            .filter(RoomEvent::is_message)
            .map(|y| y.get_concrete_unwrap(conn));
        let messages = futures::future::join_all(messages).await;

        let _ = TransmissionType::ChannelEvent(messages)
            .wrap_into_transmission()
            .send(stream)
            .await;
    }
}

pub async fn handle_get_prior(
    channel_id: i64,
    conn: &DbConn,
    stream: &mut ws::stream::DuplexStream,
    last_msg: i64,
) {
    let msg = conn.get_msg_by_id(last_msg).await;

    let message = match msg {
        Ok(x) => x,
        Err(_) => {
            let _ = TransmissionType::InvalidTransmission
                .wrap_into_transmission()
                .send(stream)
                .await;
            return;
        }
    };

    let a = conn
        .get_events_prior(channel_id, message.timestamp, last_msg, 40)
        .await;

    if let Ok(x) = a {
        if x.is_empty() {
            let _ = TransmissionType::NoMorePrior
                .wrap_into_transmission()
                .send(stream)
                .await;
        } else {
            let messages = x
                .into_iter()
                .filter(RoomEvent::is_message)
                .map(|y| y.get_concrete_unwrap(conn));
            let messages = futures::future::join_all(messages).await;
            let _ = TransmissionType::PriorMessages(messages)
                .wrap_into_transmission()
                .send(stream)
                .await;
        }
    }
}

pub async fn handle_get_server(
    server_id: i64,
    conn: &DbConn,
    stream: &mut ws::stream::DuplexStream,
) {
    let members_fut = conn.get_community_members(server_id);
    let channels_fut = conn.get_community_rooms(server_id);
    let (members, channels) = join!(members_fut, channels_fut);
    let members = members
        .unwrap_or(vec![])
        .into_iter()
        .map(ServerMember::into)
        .collect();
    let data = ServerInfoData {
        users: members,
        channels: channels
            .unwrap_or_default()
            .into_iter()
            .map(Room::into)
            .collect(),
    };

    let _ = TransmissionType::ServerInfo(data)
        .wrap_into_transmission()
        .send(stream)
        .await;
}

pub async fn handle_join_server(
    server_id: i64,
    userid: i64,
    conn: &DbConn,
    stream: &mut ws::stream::DuplexStream,
) {
    let a = conn.join_community(server_id, userid, None).await;
    let _ = TransmissionType::JoinServerResult(a)
        .wrap_into_transmission()
        .send(stream)
        .await;
}
