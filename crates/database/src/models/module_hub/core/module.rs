use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::types::JsonValue;
use sqlx::FromRow;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct MoveModule {
    pub transaction_version: i64,
    pub write_set_change_index: i64,
    pub transaction_block_height: i64,
    pub name: String,
    pub address: String,
    pub bytecode: Option<Vec<u8>>,
    pub friends: Option<JsonValue>,
    pub exposed_functions: Option<JsonValue>,
    pub structs: Option<JsonValue>,
    pub is_deleted: bool,
    pub inserted_at: NaiveDateTime,
}
