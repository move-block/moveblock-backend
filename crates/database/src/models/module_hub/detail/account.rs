use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct AccountDetail {
    pub id: i32,
    pub address: String,
    pub alias: Option<String>,
}
