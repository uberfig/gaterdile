use crate::{
    db::{DbConn, get_msg_by_id},
    // schema::db_schema,
    transmission::{
        NewTransmissionMessage, TransmissionChannel, TransmissionMessage, TransmissionServerMember,
    },
};
use serde::{Deserialize, Serialize};
use rocket_db_pools::Connection;


#[derive(
    Default,
    Deserialize,
    // Queryable,
    // Insertable,
    Debug,
    Serialize,
    Clone,
    // QueryableByName,
    // Identifiable,
    // Selectable,
)]
// #[diesel(primary_key(id))]
// #[diesel(table_name = db_schema::messages)]
pub struct Message {
    // #[diesel(deserialize_as = Option<i64>)]
    pub id: Option<i64>,
    pub sender: i64,
    pub server: i64,
    pub channel: i64,
    pub reply: Option<i64>,
    pub is_reply: bool,
    pub text: String,
    pub timestamp: i64,
}

impl Message {}

impl Message {
    pub async fn to_transmission(self, conn: &mut Connection<DbConn>) -> TransmissionMessage {
        match self.reply {
            Some(x) => {
                let mut reply_uid = -1;
                let prev = match get_msg_by_id(conn, x).await {
                    Ok(x) => {
                        let x = x.unwrap();
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

// #[derive(Deserialize, Queryable, Insertable, Debug, Serialize, Clone)]
// #[diesel(table_name = db_schema::community_members)]
pub struct ServerMember {
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

// #[derive(Deserialize, Queryable, Insertable, Debug, Serialize, Clone)]
// #[diesel(table_name = db_schema::communities)]
pub struct Community {
    id: Option<i64>,
    nickname: String,
    owner: i64,
    is_public: bool,
}

// #[derive(Deserialize, Queryable, Insertable, Debug, Serialize, Clone)]
// #[diesel(table_name = db_schema::rooms)]
pub struct Room {
    pub id: Option<i64>,
    pub server: i64,
    pub name: String,
}

impl From<Room> for TransmissionChannel {
    fn from(value: Room) -> Self {
        TransmissionChannel {
            id: value.id,
            server: value.server,
            name: value.name,
        }
    }
}

impl Room {
    pub fn to_transmission(self) -> TransmissionChannel {
        TransmissionChannel {
            id: self.id,
            server: self.server,
            name: self.name,
        }
    }
}
