use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum Visibility {
    Unspecified,
    Private,
    Protected,
    Public,
    Space,
}

impl Default for Visibility {
    fn default() -> Self {
        Self::Private
    }
}

impl Visibility {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Unspecified => "Unspecified",
            Self::Private => "Private",
            Self::Protected => "Protected",
            Self::Public => "Public",
            Self::Space => "Space",
        }
    }

    pub fn icon_name(&self) -> &'static str {
        match self {
            Self::Private => "channel-private-symbolic",
            Self::Protected => "channel-protected-symbolic",
            Self::Public => "channel-public-symbolic",
            Self::Space => "workspaces-symbolic",
            Self::Unspecified => "channel-private-symbolic",
        }
    }

    pub fn all() -> &'static [Visibility] {
        &[
            Visibility::Private,
            Visibility::Protected,
            Visibility::Public,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum MemoState {
    Unspecified,
    Normal,
    Archived,
}

impl Default for MemoState {
    fn default() -> Self {
        Self::Normal
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memo {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub state: MemoState,
    #[serde(default)]
    pub creator: String,
    #[serde(default = "default_time")]
    pub create_time: String,
    #[serde(default = "default_time")]
    pub update_time: String,
    pub content: String,
    #[serde(default)]
    pub visibility: Visibility,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub pinned: bool,
    #[serde(default)]
    pub attachments: Vec<Attachment>,
    #[serde(default)]
    pub relations: Vec<MemoRelation>,
    #[serde(default)]
    pub reactions: Vec<Reaction>,
    #[serde(default)]
    pub property: Option<MemoProperty>,
}

fn default_time() -> String {
    String::new()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoProperty {
    #[serde(default)]
    pub has_link: bool,
    #[serde(default)]
    pub has_task_list: bool,
    #[serde(default)]
    pub has_code: bool,
    #[serde(default)]
    pub has_incomplete_tasks: bool,
    #[serde(default)]
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attachment {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub filename: String,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub external_link: String,
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub size: String,
    #[serde(default)]
    pub memo: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoRelation {
    pub memo: Option<MemoRelationItem>,
    pub related_memo: Option<MemoRelationItem>,
    #[serde(default)]
    pub relation_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoRelationItem {
    pub name: String,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reaction {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub creator: String,
    #[serde(default)]
    pub reaction_type: String,
    #[serde(default)]
    pub create_time: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListMemosResponse {
    #[serde(default)]
    pub memos: Vec<Memo>,
    #[serde(default)]
    pub next_page_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMemoRequest {
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<Visibility>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateMemoRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<Visibility>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pinned: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<MemoState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct ListUsersResponse {
    #[serde(default)]
    pub users: Vec<User>,
    #[serde(default)]
    pub next_page_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct User {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub username: String,
    #[serde(default)]
    pub display_name: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub avatar_url: String,
    #[serde(default)]
    pub role: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct InstanceProfile {
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub title: String,
    #[serde(default)]
    pub owner: String,
    #[serde(default)]
    pub mode: String,
}
