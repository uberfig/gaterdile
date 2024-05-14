use rocket_db_pools::Connection;
use sqlx::Error;

use super::{
    db::DbConn,
    db_event_types::{RoomEvent, RoomEventType},
    db_types::Message,
};

pub async fn get_msg_by_id(
    conn: &mut Connection<DbConn>,
    id: i64,
) -> Result<Option<Message>, Error> {
    let user = sqlx::query_as!(Message, "select * from messages where id = $1", id)
        .fetch_optional(&mut ***conn)
        .await;

    match user {
        Ok(x) => Ok(x),
        Err(x) => Err(x),
    }
}

pub async fn send_message(conn: &mut Connection<DbConn>, message: Message) -> Result<i64, Error> {
    let timestamp = message.timestamp;
    let channel_id = message.channel;
    let server_id = message.server;

    let result = sqlx::query!(
        "INSERT INTO messages(sender, server, channel, reply, is_reply, text, timestamp) VALUES($1, $2, $3, $4, $5, $6, $7) RETURNING id",
        message.sender, message.server, message.channel, message.reply, message.is_reply, message.text, message.timestamp
    ).fetch_one(&mut ***conn)
    .await;

    match result {
        Ok(x) => {
            let _y = create_channel_event(
                conn,
                channel_id,
                server_id,
                timestamp,
                RoomEventType::NewMessage(x.id),
            )
            .await;

            return Ok(x.id);
        }
        Err(x) => return Err(x),
    }
}

pub async fn create_channel_event(
    conn: &mut Connection<DbConn>,
    channel_id: i64,
    server_id: i64,
    timestamp: i64,
    event_type: RoomEventType,
) -> Result<i64, Error> {
    let event = event_type.to_event(channel_id, server_id, timestamp);

    let result = sqlx::query!(
        "INSERT INTO room_events(channel_id, server_id, timestamp, event_type, message, reaction, creator, deleted) VALUES($1, $2, $3, $4, $5, $6, $7, $8) RETURNING id",
        event.channel_id, event.server_id, event.timestamp, event.event_type, event.message, event.reaction, event.creator, event.deleted
    ).fetch_one(&mut ***conn)
    .await;

    match result {
        Ok(x) => return Ok(x.id),
        Err(x) => return Err(x),
    }
}

/// gets all events in a room that have happened after the provided timestamp excluding the message with the provided id
pub async fn get_room_events_since_timestamp_and_id(
    conn: &mut Connection<DbConn>,
    channel_id: i64,
    since: i64,
    id: i64,
    amount: i64,
) -> Result<Vec<RoomEvent>, Error> {
    let mut a = sqlx::query_as!(
        RoomEvent,
        "SELECT * FROM room_events WHERE channel_id = $1 AND timestamp >= $2 AND id != $3 ORDER BY timestamp DESC LIMIT $4",
        channel_id, since, id, amount
    ).fetch_all(&mut ***conn).await;

    match &mut a {
        Ok(x) => {
            x.sort_unstable_by_key(|y| y.timestamp);
        }
        Err(_) => {}
    }

    a
}

/// get events that happened prior to a given event
pub async fn get_events_prior(
    conn: &mut Connection<DbConn>,
    channel_id: i64,
    prior_to: i64,
    last_msg: i64,
    amount: i64,
) -> Result<Vec<RoomEvent>, Error> {
    let mut a: Result<Vec<RoomEvent>, Error> = sqlx::query_as!(
        RoomEvent,
        "SELECT * FROM room_events WHERE channel_id = $1 AND timestamp <= $2 AND id != $3 ORDER BY timestamp ASC LIMIT $4",
        channel_id, prior_to, last_msg, amount
    ).fetch_all(&mut ***conn).await;

    match &mut a {
        Ok(x) => {
            x.sort_unstable_by_key(|y| y.timestamp);
        }
        Err(_) => {}
    }

    a
}

/// get the latest n events in a room, good for first opening a room
pub async fn get_room_events(
    conn: &mut Connection<DbConn>,
    channel_id: i64,
    amount: i64,
) -> Result<Vec<RoomEvent>, Error> {
    let mut a = sqlx::query_as!(
        RoomEvent,
        "SELECT * FROM room_events WHERE channel_id = $1 ORDER BY timestamp DESC LIMIT $2",
        channel_id,
        amount
    )
    .fetch_all(&mut ***conn)
    .await;

    match &mut a {
        Ok(x) => {
            x.sort_unstable_by_key(|y| y.timestamp);
        }
        Err(_) => {}
    }

    a
}
