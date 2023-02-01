use crate::models::module_hub::composite::function::ModuleFunctionWithDetail;
use crate::models::module_hub::core::module::MoveModule;
use crate::models::module_hub::detail::account::AccountDetail;
use crate::models::module_hub::detail::module::ModuleDetail;

use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct MoveModuleWithDetail {
    pub move_module: MoveModule,
    pub module_detail: Option<ModuleDetail>,
}

impl MoveModuleWithDetail {
    pub fn compose(move_module: MoveModule, module_detail: Option<ModuleDetail>) -> Self {
        MoveModuleWithDetail {
            move_module,
            module_detail,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct MoveModuleWithFunctionsAccountAndDetails {
    pub move_module_with_detail: MoveModuleWithDetail,
    pub account_detail: Option<AccountDetail>,
    pub functions_with_detail: Vec<ModuleFunctionWithDetail>,
}

impl MoveModuleWithFunctionsAccountAndDetails {
    pub fn compose(
        move_module_with_detail: MoveModuleWithDetail,
        account_detail: Option<AccountDetail>,
        functions_with_detail: Vec<ModuleFunctionWithDetail>,
    ) -> Self {
        MoveModuleWithFunctionsAccountAndDetails {
            move_module_with_detail,
            account_detail,
            functions_with_detail,
        }
    }
}
