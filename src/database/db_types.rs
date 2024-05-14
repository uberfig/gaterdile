use crate::{
    database::db::DbConn,
    transmission::{NewTransmissionMessage, TransmissionMessage},
};
use rocket_db_pools::Connection;
use serde::{Deserialize, Serialize};

use super::messages::get_msg_by_id;

#[derive(Default, Deserialize, Debug, Serialize, Clone)]
pub struct Message {
    pub id: Option<i64>,
    pub sender: i64,
    pub server: i64,
    pub channel: i64,
    pub reply: Option<i64>,
    pub is_reply: bool,
    pub text: String,
    pub timestamp: i64,
}

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
