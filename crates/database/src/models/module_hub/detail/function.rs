use serde::{Deserialize, Serialize};
use sqlx::types::JsonValue;
use sqlx::FromRow;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct ModuleFunctionDetail {
    pub id: i32,
    pub address: String,
    pub module_name: String,
    pub function_name: String,
    pub description: Option<String>,
    pub param_names: Option<JsonValue>,
    pub generic_type_params: Option<JsonValue>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewModuleFunctionDetail {
    pub address: String,
    pub module_name: String,
    pub function_name: String,
    pub description: Option<String>,
    pub param_names: Option<JsonValue>,
    pub generic_type_params: Option<JsonValue>,
}

impl NewModuleFunctionDetail {
    pub fn new(
        address: &str,
        module_name: &str,
        function_name: &str,
        description: Option<String>,
        param_names: Option<Vec<String>>,
        generic_type_params: Option<Vec<String>>,
    ) -> Self {
        NewModuleFunctionDetail {
            address: address.to_string(),
            module_name: module_name.to_string(),
            function_name: function_name.to_string(),
            description,
            param_names: Some(JsonValue::from(param_names)),
            generic_type_params: Some(JsonValue::from(generic_type_params)),
        }
    }
}
