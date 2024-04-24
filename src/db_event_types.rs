use crate::{
    db::{get_msg_by_id, DbConn},
    db_types::Message,
    transmission,
};
// use diesel::result::Error;
// use serde::{Deserialize, Serialize};
use rocket_db_pools::Connection;
use sqlx::Error;

// #[derive(Deserialize, Queryable, Insertable, Debug, Serialize, Clone)]
// #[diesel(table_name = db_schema::room_events)]
#[derive(Debug)]
pub struct RoomEvent {
    pub id: Option<i64>,
    pub channel_id: i64,
    pub server_id: i64,
    pub timestamp: i64,
    pub event_type: i32,
    pub message: Option<i64>,
    pub reaction: Option<i64>,
    pub creator: Option<i64>,
    pub deleted: Option<i64>, //used for the id of deleted content
}

pub enum RoomEventType {
    NewMessage(i64),
    MessageDeleted(i64),
    NewReaction(i64),
    DeleteReaction(i64),
    UserJoin(i64),
    UserLeave(i64),
    Error,
}
impl Default for RoomEvent {
    fn default() -> Self {
        RoomEvent {
            id: None,
            channel_id: -1,
            server_id: -1,
            timestamp: -1,
            event_type: -1,
            message: None,
            reaction: None,
            creator: None,
            deleted: None,
        }
    }
}

impl RoomEvent {
    fn to_event_type(&self) -> RoomEventType {
        match self.event_type {
            0 => RoomEventType::NewMessage(self.message.unwrap()),
            1 => RoomEventType::MessageDeleted(self.deleted.unwrap()),
            2 => RoomEventType::NewReaction(self.reaction.unwrap()),
            3 => RoomEventType::DeleteReaction(self.deleted.unwrap()),
            4 => RoomEventType::UserJoin(self.creator.unwrap()),
            5 => RoomEventType::UserLeave(self.creator.unwrap()),
            _ => RoomEventType::Error,
        }
    }
    pub fn is_message(&self) -> bool {
        self.event_type == 0
    }
    pub async fn get_message(self, conn: &mut Connection<DbConn>) -> Message {
        get_msg_by_id(
            conn,
            self.message
                .expect("tried to get message when msg id is none"),
        )
        .await
        .unwrap()
        .unwrap()
    }
    pub async fn get_concrete(
        self,
        conn: &mut Connection<DbConn>,
    ) -> Result<transmission::ChannelEvent, Error> {
        let evt_type = self.to_event_type();
        match evt_type {
            RoomEventType::NewMessage(x) => {
                let msg = get_msg_by_id(conn, x).await;
                match msg {
                    Ok(y) => {
                        let evt = transmission::ChannelEventType::NewMessage(
                            y.unwrap().to_transmission(conn).await,
                        );
                        Ok(transmission::ChannelEvent {
                            event_type: evt.to_string(),
                            data: evt,
                        })
                    }
                    Err(y) => Err(y),
                }
            }
            RoomEventType::MessageDeleted(_) => todo!(),
            RoomEventType::NewReaction(_) => todo!(),
            RoomEventType::DeleteReaction(_) => todo!(),
            RoomEventType::UserJoin(_) => todo!(),
            RoomEventType::UserLeave(_) => todo!(),
            RoomEventType::Error => todo!(),
        }
    }
    pub async fn get_concrete_unwrap(
        self,
        conn: &mut Connection<DbConn>,
    ) -> transmission::ChannelEvent {
        self.get_concrete(conn).await.unwrap()
    }
}

impl RoomEventType {
    fn to_int(&self) -> i32 {
        match self {
            RoomEventType::NewMessage(_) => 0,
            RoomEventType::MessageDeleted(_) => 1,
            RoomEventType::NewReaction(_) => 2,
            RoomEventType::DeleteReaction(_) => 3,
            RoomEventType::UserJoin(_) => 4,
            RoomEventType::UserLeave(_) => 5,
            RoomEventType::Error => -1,
        }
    }
    pub fn to_event(&self, channel_id: i64, server_id: i64, timestamp: i64) -> RoomEvent {
        match self {
            RoomEventType::NewMessage(x) => RoomEvent {
                channel_id,
                server_id,
                timestamp,
                event_type: self.to_int(),
                message: Some(*x),
                ..Default::default()
            },
            RoomEventType::MessageDeleted(x) => RoomEvent {
                channel_id,
                server_id,
                timestamp,
                event_type: self.to_int(),
                deleted: Some(*x),
                ..Default::default()
            },
            RoomEventType::NewReaction(x) => RoomEvent {
                channel_id,
                server_id,
                timestamp,
                event_type: self.to_int(),
                reaction: Some(*x),
                ..Default::default()
            },
            RoomEventType::DeleteReaction(x) => RoomEvent {
                channel_id,
                server_id,
                timestamp,
                event_type: self.to_int(),
                deleted: Some(*x),
                ..Default::default()
            },
            RoomEventType::UserJoin(x) | RoomEventType::UserLeave(x) => RoomEvent {
                channel_id,
                server_id,
                timestamp,
                event_type: self.to_int(),
                creator: Some(*x),
                ..Default::default()
            },
            RoomEventType::Error => RoomEvent {
                channel_id,
                server_id,
                timestamp,
                event_type: self.to_int(),
                ..Default::default()
            },
        }
    }
}
