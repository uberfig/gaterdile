use rocket::{
    futures::SinkExt,
    serde::{Deserialize, Serialize},
};
use rocket_ws as ws;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TransmissionChannel {
    pub id: Option<i64>,
    pub server: i64,
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TransmissionCommunity {
    id: Option<i64>,
    nickname: String,
    owner: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum InsertError {
    Success(usize),
    UsernameTaken,
    DbError,
    InvalidPassword,
    InvalidUsername,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum AuthErr {
    Success(i64),
    InvalidUsername,
    InvalidPassword,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum JoinServerResult {
    Success(i64),
    AlreadyInServer,
    NotAuthorised,
    Failure,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TransmissionServerMember {
    pub server_id: i64,
    pub userid: i64,
    pub nickname: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserAuth {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct NewMessage {
    text: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct React {
    pub id: i32,
    pub message_id: i32,
    pub user_id: i32,
    pub emoji: i32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChannelInfo {
    pub messages: Vec<TransmissionMessage>,
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
pub struct NewTransmissionMessage {
    pub server: i64,
    pub channel: i64,
    pub reply: Option<i64>,
    pub text: String,
    // pub contents: Vec<MessageContent>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TransmissionMessage {
    pub id: Option<i64>,
    pub sender: i64,
    pub server: i64,
    pub channel: i64,
    pub reply: Option<i64>,
    pub is_reply: bool,
    pub reply_prev: Option<String>,
    pub reply_uid: Option<i64>,
    pub text: String,
    pub timestamp: i64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerInfoData {
    pub users: Vec<TransmissionServerMember>,
    pub channels: Vec<TransmissionChannel>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChannelEvent {
    pub event_type: String,
    pub data: ChannelEventType,
}

#[derive(Debug, Deserialize, Serialize, Clone)]

pub enum ChannelEventType {
    NewMessage(TransmissionMessage),
    MessageDeleted(i32),
    NewReaction(React),
    DeleteReaction(i32),
    Error,
}

impl std::fmt::Display for ChannelEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ChannelEventType::NewMessage(_) => write!(f, "NewMessage"),
            ChannelEventType::MessageDeleted(_) => write!(f, "MessageDeleted"),
            ChannelEventType::NewReaction(_) => write!(f, "NewReaction"),
            ChannelEventType::DeleteReaction(_) => write!(f, "DeleteReaction"),
            ChannelEventType::Error => write!(f, "Error"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerEvent {
    event_type: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum ServerEventType {
    UserJoin(i32),
    UserLeave(i32),
    Error,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct UserEvent {
    event_type: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum TransmissionType {
    SendMessage(NewTransmissionMessage),
    Reaction(React),
    Auth(UserAuth),

    CreateUser(UserAuth),

    GetCommunity(i64), //requests to get server info
    GetUserServers,
    JoinCommunity(i64),
    CreateCommunity(String), //create server with given nickname
    GetRoom(i64, i64),       //server, channel gets the channels recent messages
    CreateRoom(i64, String), //request to create a room in provided server with provided nickname

    GetPriorMessages(i64), //get messages prior to provided value
    GetEmoji(i64),
    GetAttachment(i64),
    //from server only:
    InvalidTransmission,
    RequestAuth,
    AuthResult(AuthErr),
    CreateUserResult(InsertError),
    ServerInfo(ServerInfoData),
    UserCommunities(Vec<TransmissionCommunity>),
    JoinServerResult(JoinServerResult),
    PriorMessages(Vec<ChannelEvent>),
    NoMorePrior,

    ChannelEvent(Vec<ChannelEvent>),
    ServerEvent(Vec<ServerEvent>),
    UserEvent(Vec<UserEvent>),
}

impl std::fmt::Display for TransmissionType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TransmissionType::SendMessage(_) => write!(f, "SendMessage"),
            TransmissionType::Reaction(_) => write!(f, "Reaction"),
            TransmissionType::RequestAuth => write!(f, "RequestAuth"),
            TransmissionType::Auth(_) => write!(f, "Auth"),
            TransmissionType::AuthResult(_) => write!(f, "AuthResult"),
            TransmissionType::GetCommunity(_) => write!(f, "GetServer"),
            TransmissionType::GetRoom(..) => write!(f, "GetChannel"),
            TransmissionType::CreateUser(_) => write!(f, "CreateUser"),
            TransmissionType::InvalidTransmission => write!(f, "InvalidTransmission"),
            TransmissionType::CreateUserResult(_) => write!(f, "CreateUserResult"),
            TransmissionType::ServerInfo(_) => write!(f, "ServerInfo"),
            TransmissionType::GetUserServers => write!(f, "GetUserServers"),
            TransmissionType::UserCommunities(_) => write!(f, "UserServers"),
            TransmissionType::JoinCommunity(_) => write!(f, "JoinServer"),
            TransmissionType::JoinServerResult(_) => write!(f, "JoinServerResult"),
            TransmissionType::GetPriorMessages(_) => write!(f, "GetPriorMessages"),
            TransmissionType::PriorMessages(_) => write!(f, "PriorMessages"),
            TransmissionType::NoMorePrior => write!(f, "NoMorePrior"),
            TransmissionType::GetEmoji(_) => write!(f, "GetEmoji"),
            TransmissionType::GetAttachment(_) => write!(f, "GetAttachment"),
            TransmissionType::ChannelEvent(_) => write!(f, "ChannelEvent"),
            TransmissionType::ServerEvent(_) => write!(f, "ServerEvent"),
            TransmissionType::UserEvent(_) => write!(f, "UserEvent"),
            TransmissionType::CreateCommunity(_) => write!(f, "CreateCommunity"),
            TransmissionType::CreateRoom(..) => write!(f, "CreateRoom"),
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
    pub fn parse(val: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str::<Transmission>(val)
    }
    pub fn invalid() -> Transmission {
        Transmission {
            data: TransmissionType::InvalidTransmission,
            transmission_type: TransmissionType::InvalidTransmission.to_string(),
        }
    }
    pub async fn send(
        &self,
        stream: &mut ws::stream::DuplexStream,
    ) -> Result<(), ws::result::Error> {
        stream
            .send(rocket_ws::Message::Text(self.stringify()))
            .await
    }
}
