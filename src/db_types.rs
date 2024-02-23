use crate::{
    db::DbConn,
    schema::db_schema,
    transmission::{self, NewTransmissionMessage, TransmissionChannel, TransmissionMessage, TransmissionServerMember},
};
use diesel::result::Error;
use serde::{Deserialize, Serialize};

#[derive(
    Default,
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
// #[diesel(check_for_backend(diesel::post))]
pub struct Message {
    // #[diesel(deserialize_as = "i64")]
    #[diesel(deserialize_as = Option<i64>)]
    pub id: Option<i64>,
    pub sender: i64,
    pub server: i64,
    pub channel: i64,
    pub reply: Option<i64>,
    pub is_reply: bool,
    pub text: String,
    // pub emoji: Option<Vec<u8>>,
    pub timestamp: i64,
}

impl Message {}

impl Message {
    pub async fn to_transmission(self, conn: &DbConn) -> TransmissionMessage {
        match self.reply {
            Some(x) => {
                let mut reply_uid = -1;
                let prev = match conn.get_msg_by_id(x).await {
                    Ok(x) => {
                        reply_uid = x.sender;
                        x.text
                    }
                    Err(_) => "Message Deleted".to_string(),
                };
                TransmissionMessage {
                    id: self.id,
                    sender: self.sender,
                    server: self.server,
                    channel: self.channel,
                    reply: self.reply,
                    is_reply: self.is_reply,
                    reply_prev: Some(prev),
                    reply_uid: Some(reply_uid),
                    text: self.text,
                    timestamp: self.timestamp,
                }
            }
            None => TransmissionMessage {
                id: self.id,
                sender: self.sender,
                server: self.server,
                channel: self.channel,
                reply: self.reply,
                is_reply: self.is_reply,
                reply_prev: None,
                reply_uid: None,
                text: self.text,
                timestamp: self.timestamp,
            },
        }
        //
    }
    pub fn from_newmsg(value: NewTransmissionMessage, uid: i64) -> Self {
        use std::time::SystemTime;
        Message {
            id: None,
            sender: uid,
            server: value.server,
            channel: value.channel,
            reply: value.reply,
            is_reply: value.reply.is_some(),
            text: value.text,
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64,
        }
    }
}

#[derive(Deserialize, Queryable, Insertable, Debug, Serialize, Clone)]
#[diesel(table_name = db_schema::server_members)]
pub struct ServerMember {
    // pub id: Option<i32>,
    pub server_id: i64,
    pub userid: i64,
    pub nickname: Option<String>,
}

impl From<ServerMember> for TransmissionServerMember {
    fn from(value: ServerMember) -> Self {
        TransmissionServerMember {
            server_id: value.server_id,
            userid: value.userid,
            nickname: value.nickname,
        }
    }
}

#[derive(Deserialize, Queryable, Insertable, Debug, Serialize, Clone)]
#[diesel(table_name = db_schema::channels)]
pub struct Channel {
    pub id: Option<i64>,
    pub server: i64,
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

pub enum ChannelEventType {
    NewMessage(i64),
    MessageDeleted(i64),
    NewReaction(i64),
    DeleteReaction(i64),
    UserJoin(i64),
    UserLeave(i64),
    Error,
}

impl ChannelEvent {
    fn to_event_type(&self) -> ChannelEventType {
        match self.event_type {
            0 => ChannelEventType::NewMessage(self.message.unwrap()),
            1 => ChannelEventType::MessageDeleted(self.deleted.unwrap()),
            2 => ChannelEventType::NewReaction(self.reaction.unwrap()),
            3 => ChannelEventType::DeleteReaction(self.deleted.unwrap()),
            4 => ChannelEventType::UserJoin(self.creator.unwrap()),
            5 => ChannelEventType::UserLeave(self.creator.unwrap()),
            _ => ChannelEventType::Error,
        }
    }
    pub fn is_message(&self) -> bool {
        self.event_type == 0
    }
    pub async fn get_message(self, conn: &DbConn) -> Message {
        conn.get_msg_by_id(
            self.message
                .expect("tried to get message when msg id is none"),
        )
        .await
        .unwrap()
    }
    pub async fn get_concrete(self, conn: &DbConn) -> Result<transmission::ChannelEvent, Error> {
        let evt_type = self.to_event_type();
        match evt_type {
            ChannelEventType::NewMessage(x) => {
                let msg = conn.get_msg_by_id(x).await;
                match msg {
                    Ok(y) => {
                        let evt = transmission::ChannelEventType::NewMessage(
                            y.to_transmission(conn).await,
                        );
                        Ok(transmission::ChannelEvent {
                            event_type: evt.to_string(),
                            data: evt,
                        })
                    }
                    Err(y) => Err(y),
                }
            }
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
    pub fn to_event(&self, channel_id: i64, server_id: i64, timestamp: i64) -> ChannelEvent {
        match self {
            ChannelEventType::NewMessage(x) => ChannelEvent {
                id: None,
                channel_id,
                server_id,
                timestamp,
                event_type: self.to_int(),
                message: Some(*x),
                reaction: None,
                creator: None,
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
                creator: None,
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
                creator: None,
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
                creator: None,
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
                creator: Some(*x),
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
                creator: Some(*x),
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
                creator: None,
                deleted: None,
            },
        }
    }
}
