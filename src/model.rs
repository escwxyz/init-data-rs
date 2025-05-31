use serde_with::DisplayFromStr;
use serde_with::serde_as;
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

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitData {
    pub auth_date: u64,
    pub can_send_after: Option<u32>,
    pub chat: Option<Chat>,
    pub chat_type: Option<ChatType>,
    pub chat_instance: Option<i64>,
    pub data: Option<String>,
    pub hash: String,
    pub query_id: Option<String>,
    pub receiver: Option<User>,
    pub start_param: Option<String>,
    pub user: Option<User>,
    pub signature: String
}
