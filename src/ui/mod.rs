pub mod app;
pub mod header;
pub mod markdown;
pub mod memo_editor;
pub mod memo_row;
pub mod settings_dialog;

use crate::api::Visibility;
use chrono::Local;

pub const MAX_PAGE_SIZE: u32 = 100;

pub fn visibility_from_index(index: usize) -> Visibility {
    Visibility::all()
        .get(index)
        .cloned()
        .unwrap_or(Visibility::Private)
}

pub fn index_from_visibility(visibility: &Visibility) -> usize {
    Visibility::all()
        .iter()
        .position(|v| v == visibility)
        .unwrap_or(0)
}

pub fn format_time(rfc3339: &str) -> String {
    let Ok(dt) = chrono::DateTime::parse_from_rfc3339(rfc3339) else {
        return rfc3339.to_string();
    };
    dt.with_timezone(&Local)
        .format("%Y-%m-%d %H:%M")
        .to_string()
}

pub fn memo_id(name: &str) -> String {
    name.strip_prefix("memos/").unwrap_or(name).to_string()
}