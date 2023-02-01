use crate::models::module_hub::core::function::ModuleFunction;

use crate::models::module_hub::detail::account::AccountDetail;
use crate::models::module_hub::detail::function::ModuleFunctionDetail;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct ModuleFunctionWithAccountDetail {
    pub module_function: ModuleFunction,
    pub account_detail: Option<AccountDetail>,
}

impl ModuleFunctionWithAccountDetail {
    pub fn compose(module_function: ModuleFunction, account_detail: Option<AccountDetail>) -> Self {
        ModuleFunctionWithAccountDetail {
            module_function,
            account_detail,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct ModuleFunctionWithDetail {
    pub module_function: ModuleFunction,
    pub function_detail: Option<ModuleFunctionDetail>,
}

impl ModuleFunctionWithDetail {
    pub fn compose(
        module_function: ModuleFunction,
        function_detail: Option<ModuleFunctionDetail>,
    ) -> Self {
        ModuleFunctionWithDetail {
            module_function,
            function_detail,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromRow)]
pub struct ModuleFunctionWithOwnAndAccountDetail {
    pub module_function: ModuleFunction,
    pub account_detail: Option<AccountDetail>,
    pub function_detail: Option<ModuleFunctionDetail>,
}

impl ModuleFunctionWithOwnAndAccountDetail {
    pub fn compose(
        module_function: ModuleFunction,
        account_detail: Option<AccountDetail>,
        function_detail: Option<ModuleFunctionDetail>,
    ) -> Self {
        ModuleFunctionWithOwnAndAccountDetail {
            module_function,
            account_detail,
            function_detail,
        }
    }
}
