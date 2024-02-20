use crate::{
    db::DbConn,
    db_types::{Channel, ChannelEvent},
    transmission::{ServerInfoData, TransmissionType},
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
            .filter(ChannelEvent::is_message)
            .map(|y| y.get_concrete_unwrap(conn));
        let messages = futures::future::join_all(messages).await;
        // let messages = x.into_iter().filter(ChannelEvent::is_message).map(|y| y.get_message(conn));
        // let messages = futures::future::join_all(messages).await;

        let _ = TransmissionType::ChannelEvent(messages)
            .wrap_into_transmission()
            .send(stream)
            .await;
    }
}

pub async fn handle_get_prior(
    server_id: i64,
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
        .get_events_prior(server_id, message.timestamp, last_msg, 40)
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
                .filter(ChannelEvent::is_message)
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
    let members_fut = conn.get_server_members(server_id);
    let channels_fut = conn.get_server_channels(server_id);
    let (members, channels) = join!(members_fut, channels_fut);
    let data = ServerInfoData {
        users: members.unwrap_or(vec![]),
        channels: channels
            .unwrap_or_default()
            .into_iter()
            .map(Channel::into)
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
    let a = conn.join_server(server_id, userid, None).await;
    let _ = TransmissionType::JoinServerResult(a)
        .wrap_into_transmission()
        .send(stream)
        .await;
}
