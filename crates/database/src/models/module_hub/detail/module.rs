use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct ModuleDetail {
    pub id: i32,
    pub description: Option<String>,
    pub github_url: Option<String>,
    pub rev: Option<String>,
    pub subdir: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct NewModuleDetail {
    pub address: String,
    pub module_name: String,
    pub description: Option<String>,
    pub github_url: Option<String>,
    pub rev: Option<String>,
    pub subdir: Option<String>,
}

impl NewModuleDetail {
    pub fn new(
        address: &str,
        module_name: &str,
        description: &Option<String>,
        github_url: &Option<String>,
        rev: &Option<String>,
        subdir: &Option<String>,
    ) -> Self {
        NewModuleDetail {
            address: address.to_string(),
            module_name: module_name.to_string(),
            description: description.clone(),
            github_url: github_url.clone(),
            rev: rev.clone(),
            subdir: subdir.clone(),
        }
    }
}
