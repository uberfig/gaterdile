use crate::{
    db::DbConn, schema::db_schema, transmission::{self, TransmissionChannel}
};
use diesel::result::Error;
use serde::{Deserialize, Serialize};

#[derive(
    Deserialize,
    Queryable,
    Insertable,
    Debug,
    Serialize,
    Clone,
    QueryableByName,
    Identifiable,
    Selectable,
)]
#[diesel(primary_key(id))]
#[diesel(table_name = db_schema::messages)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Message {
    pub id: Option<i32>,
    pub sender: i32,
    pub server: i32,
    pub channel: i32,
    pub reply: Option<i32>,
    pub text: String,
    // pub emoji: Option<Vec<u8>>,
    pub timestamp: i64,
}

#[derive(Deserialize, Queryable, Insertable, Debug, Serialize, Clone)]
#[diesel(table_name = db_schema::server_members)]
pub struct ServerMember {
    // pub id: Option<i32>,
    pub server_id: i32,
    pub userid: i32,
    pub nickname: Option<String>,
}

#[derive(Deserialize, Queryable, Insertable, Debug, Serialize, Clone)]
#[diesel(table_name = db_schema::channels)]
pub struct Channel {
    pub id: Option<i32>,
    pub server: i32,
    pub name: String,
}

impl From<Channel> for TransmissionChannel {
    fn from(value: Channel) -> Self {
        TransmissionChannel {
            id: value.id,
            server: value.server,
            name: value.name,
        }
    }
}

impl Channel {
    pub fn to_transmission(self) -> TransmissionChannel {
        TransmissionChannel {
            id: self.id,
            server: self.server,
            name: self.name,
        }
    }
}

#[derive(Deserialize, Queryable, Insertable, Debug, Serialize, Clone)]
#[diesel(table_name = db_schema::channel_events)]
pub struct ChannelEvent {
    pub id: Option<i32>,
    pub channel_id: i32,
    pub server_id: i32,
    pub timestamp: i64,
    pub event_type: i32,
    pub message: Option<i32>,
    pub reaction: Option<i32>,
    pub user: Option<i32>,
    pub deleted: Option<i32>, //used for the id of deleted content
}

pub enum ChannelEventType {
    NewMessage(i32),
    MessageDeleted(i32),
    NewReaction(i32),
    DeleteReaction(i32),
    UserJoin(i32),
    UserLeave(i32),
    Error,
}

impl ChannelEvent {
    fn to_event_type(&self) -> ChannelEventType {
        match self.event_type {
            0 => ChannelEventType::NewMessage(self.message.unwrap()),
            1 => ChannelEventType::MessageDeleted(self.deleted.unwrap()),
            2 => ChannelEventType::NewReaction(self.reaction.unwrap()),
            3 => ChannelEventType::DeleteReaction(self.deleted.unwrap()),
            4 => ChannelEventType::UserJoin(self.user.unwrap()),
            5 => ChannelEventType::UserLeave(self.user.unwrap()),
            _ => ChannelEventType::Error,
        }
    }
    pub fn is_message(&self) -> bool {
        self.event_type == 0
    }
    pub async fn get_message(self, conn: &DbConn) -> Message {
        conn.get_msg_by_id(self.message.expect("tried to get message when msg id is none")).await.unwrap()
    }
    pub async fn get_concrete(self, conn: &DbConn) -> Result<transmission::ChannelEvent, Error> {
        let evt_type = self.to_event_type();
        match evt_type {
            ChannelEventType::NewMessage(x) => {
                let msg = conn.get_msg_by_id(x).await;
                match msg {
                    Ok(y) => {
                        let evt = transmission::ChannelEventType::NewMessage(y);
                        Ok(transmission::ChannelEvent { event_type: evt.to_string(), data: evt })
                    },
                    Err(y) => Err(y),
                }                
            },
            ChannelEventType::MessageDeleted(_) => todo!(),
            ChannelEventType::NewReaction(_) => todo!(),
            ChannelEventType::DeleteReaction(_) => todo!(),
            ChannelEventType::UserJoin(_) => todo!(),
            ChannelEventType::UserLeave(_) => todo!(),
            ChannelEventType::Error => todo!(),
        }
    }
    pub async fn get_concrete_unwrap(self, conn: &DbConn) -> transmission::ChannelEvent {
        self.get_concrete(conn).await.unwrap()
    }
}

impl ChannelEventType {
    fn to_int(&self) -> i32 {
        match self {
            ChannelEventType::NewMessage(_) => 0,
            ChannelEventType::MessageDeleted(_) => 1,
            ChannelEventType::NewReaction(_) => 2,
            ChannelEventType::DeleteReaction(_) => 3,
            ChannelEventType::UserJoin(_) => 4,
            ChannelEventType::UserLeave(_) => 5,
            ChannelEventType::Error => -1,
        }
    }
    pub fn to_event(&self, channel_id: i32, server_id:i32, timestamp: i64) -> ChannelEvent {
        match self {
            ChannelEventType::NewMessage(x) => ChannelEvent {
                id: None,
                channel_id,
                server_id,
                timestamp,
                event_type: self.to_int(),
                message: Some(*x),
                reaction: None,
                user: None,
                deleted: None,
            },
            ChannelEventType::MessageDeleted(x) => ChannelEvent {
                id: None,
                channel_id,
                server_id,
                timestamp,
                event_type: self.to_int(),
                message: None,
                reaction: None,
                user: None,
                deleted: Some(*x),
            },
            ChannelEventType::NewReaction(x) => ChannelEvent {
                id: None,
                channel_id,
                server_id,
                timestamp,
                event_type: self.to_int(),
                message: None,
                reaction: Some(*x),
                user: None,
                deleted: None,
            },
            ChannelEventType::DeleteReaction(x) => ChannelEvent {
                id: None,
                channel_id,
                server_id,
                timestamp,
                event_type: self.to_int(),
                message: None,
                reaction: None,
                user: None,
                deleted: Some(*x),
            },
            ChannelEventType::UserJoin(x) => ChannelEvent {
                id: None,
                channel_id,
                server_id,
                timestamp,
                event_type: self.to_int(),
                message: None,
                reaction: None,
                user: Some(*x),
                deleted: None,
            },
            ChannelEventType::UserLeave(x) => ChannelEvent {
                id: None,
                channel_id,
                server_id,
                timestamp,
                event_type: self.to_int(),
                message: None,
                reaction: None,
                user: Some(*x),
                deleted: None,
            },
            ChannelEventType::Error => ChannelEvent {
                id: None,
                channel_id,
                server_id,
                timestamp,
                event_type: self.to_int(),
                message: None,
                reaction: None,
                user: None,
                deleted: None,
            },
        }
    }
}
