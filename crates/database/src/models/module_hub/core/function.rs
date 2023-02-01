use serde::{Deserialize, Serialize};
use sqlx::types::JsonValue;
use sqlx::FromRow;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct ModuleFunction {
    pub id: i32,
    pub module_address: String,
    pub module_name: String,
    pub move_modules_transaction_version: i64,
    pub move_modules_write_set_change_index: i64,
    pub name: String,
    pub visibility: String,
    pub is_entry: bool,
    pub generic_type_params: Option<JsonValue>,
    pub params: Option<JsonValue>,
    pub return_types: Option<JsonValue>,
}
