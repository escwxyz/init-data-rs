use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChatType {
    Sender,
    Private,
    Group,
    Supergroup,
    Channel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub added_to_attachment_menu: Option<bool>,
    pub allows_write_to_pm: Option<bool>,
    pub first_name: String,
    pub id: i64,
    pub is_bot: Option<bool>,
    pub is_premium: Option<bool>,
    pub last_name: Option<String>,
    pub language_code: Option<String>,
    pub photo_url: Option<String>,
    pub username: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chat {
    pub id: i64,
    pub photo_url: Option<String>,
    #[serde(rename = "type")]
    pub chat_type: ChatType,
    pub title: String,
    pub username: Option<String>,
}

/// This object contains data that is transferred to the Mini App when it is opened. It is empty if the Mini App was launched from a keyboard button or from inline mode.
/// See: https://core.telegram.org/bots/webapps#webappinitdata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitData {
    /// Unix time when the form was opened.
    pub auth_date: u64,
    /// Optional.
    /// Time in seconds, after which a message can be sent via the answerWebAppQuery method.
    pub can_send_after: Option<u32>,
    /// Optional.
    /// An object containing data about the chat where the bot was launched via the attachment menu. Returned for supergroups, channels and group chats – only for Mini Apps launched via the attachment menu.
    pub chat: Option<Chat>,
    /// Optional.
    /// Type of the chat from which the Mini App was opened. Can be either “sender” for a private chat with the user opening the link, “private”, “group”, “supergroup”, or “channel”. Returned only for Mini Apps launched from direct links.
    pub chat_type: Option<ChatType>,
    /// Optional.
    /// Global identifier, uniquely corresponding to the chat from which the Mini App was opened. Returned only for Mini Apps launched from a direct link.
    pub chat_instance: Option<i64>,
    /// A hash of all passed parameters, which the bot server can use to check their validity.
    pub hash: String,
    /// Optional.
    /// A unique identifier for the Mini App session, required for sending messages via the answerWebAppQuery method.
    pub query_id: Option<String>,
    /// Optional.
    /// An object containing data about the chat partner of the current user in the chat where the bot was launched via the attachment menu. Returned only for private chats and only for Mini Apps launched via the attachment menu.
    pub receiver: Option<User>,
    /// Optional.
    /// The value of the startattach parameter, passed via link. Only returned for Mini Apps when launched from the attachment menu via link.
    /// The value of the start_param parameter will also be passed in the GET-parameter tgWebAppStartParam, so the Mini App can load the correct interface right away.
    pub start_param: Option<String>,
    /// Optional. An object containing data about the current user.
    pub user: Option<User>,
    /// A signature of all passed parameters (except hash), which the third party can use to check their validity.
    /// This field is only for third-party validation, shall be optional?
    pub signature: Option<String>,
}
