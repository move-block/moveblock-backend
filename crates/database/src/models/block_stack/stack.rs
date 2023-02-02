use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::types::JsonValue;
use sqlx::FromRow;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct BlockStack {
    pub id: i32,
    pub address: String,
    pub name: String,
    pub stack: JsonValue,
    pub last_edit_datetime: NaiveDateTime,
    pub bytecode: Option<Vec<u8>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct NewBlockStack {
    pub name: String,
    pub stack: JsonValue,
}
