use rocket::{serde::{Deserialize, Serialize}, futures::SinkExt};
use crate::db::{Message, JoinServerResult, ServerMember, InsertError, AuthErr, UserAuth, Channel};
use rocket_ws as ws;

#[derive(Debug, Deserialize, Serialize)]
struct NewMessage {
    text: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct React {
    pub reaction: String,
    pub message_id: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChannelInfo {
    pub messages: Vec<Message>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MessageEmoji {
    pub id: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MessageContent {
    pub contype: String,
    pub data: MessageContentType,
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum MessageContentType {
    Text(String),
    Emoji(React),
    Mention(MessageEmoji),
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TransmissionMessage {
    pub server: i32,
    pub channel: i32,
    pub reply: Option<i32>,
    pub text: String,
    // pub contents: Vec<MessageContent>,
}

impl TransmissionMessage {
    pub fn to_message(&self, uid: i32) -> Message {
        use std::time::SystemTime;
        Message {
            id: None,
            sender: uid,
            server: self.server,
            channel: self.channel,
            reply: self.reply,
            text: self.text.clone(),
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerInfoData {
    pub users: Vec<ServerMember>,
    pub channels: Vec<Channel>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum TransmissionType {
    SendMessage(TransmissionMessage),
    Reaction(React),
    Auth(UserAuth),
    GetServer(i32),       //requests to get server info
    GetChannel(i32, i32), //server, channel gets the channels recent messages
    CreateUser(UserAuth),
    GetUserServers,
    JoinServer(i32),
    //from server only:
    InvalidTransmission,
    NewMessages(Vec<Message>),
    RequestAuth,
    AuthResult(AuthErr),
    CreateUserResult(InsertError),
    ServerInfo(ServerInfoData),
    UserServers(Vec<ServerMember>),
    JoinServerResult(JoinServerResult),
}

impl std::fmt::Display for TransmissionType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TransmissionType::NewMessages(_) => write!(f, "NewMessages"),
            TransmissionType::SendMessage(_) => write!(f, "SendMessage"),
            TransmissionType::Reaction(_) => write!(f, "Reaction"),
            TransmissionType::RequestAuth => write!(f, "RequestAuth"),
            TransmissionType::Auth(_) => write!(f, "Auth"),
            TransmissionType::AuthResult(_) => write!(f, "AuthResult"),
            TransmissionType::GetServer(_) => write!(f, "GetServer"),
            TransmissionType::GetChannel(..) => write!(f, "GetChannel"),
            TransmissionType::CreateUser(_) => write!(f, "CreateUser"),
            TransmissionType::InvalidTransmission => write!(f, "InvalidTransmission"),
            TransmissionType::CreateUserResult(_) => write!(f, "CreateUserResult"),
            TransmissionType::ServerInfo(_) => write!(f, "ServerInfo"),
            TransmissionType::GetUserServers => write!(f, "GetUserServers"),
            TransmissionType::UserServers(_) => write!(f, "UserServers"),
            TransmissionType::JoinServer(_) => write!(f, "JoinServer"),
            TransmissionType::JoinServerResult(_) => write!(f, "JoinServerResult"),
        }
    }
}

impl TransmissionType {
    pub fn wrap_into_transmission(self) -> Transmission {
        let name = self.to_string();
        Transmission {
            data: self,
            transmission_type: name,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Transmission {
    pub data: TransmissionType,
    pub transmission_type: String,
}

impl Transmission {
    pub fn stringify(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
    pub fn parse(val: &str) -> Result<Self, ()> {
        let a = serde_json::from_str::<Transmission>(val);
        match a {
            Ok(x) => return Result::Ok(x),
            Err(_x) => return Result::Err(()),
        }
    }
    pub fn invalid() -> Transmission {
        Transmission { data: TransmissionType::InvalidTransmission, transmission_type: TransmissionType::InvalidTransmission.to_string() }
    }
    pub async fn send(
        &self,
        stream: &mut ws::stream::DuplexStream,
    ) -> Result<(), ws::result::Error> {
        let _a = stream
            .send(rocket_ws::Message::Text(self.stringify()))
            .await;
        _a
    }
}