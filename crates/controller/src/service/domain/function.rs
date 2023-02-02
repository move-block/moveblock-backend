use crate::service::{
    domain::{account, Count},
    Error,
};

use database::db::PostgresPool;
use database::models::module_hub::composite::function::{
    ModuleFunctionWithAccountDetail, ModuleFunctionWithDetail,
    ModuleFunctionWithOwnAndAccountDetail,
};
use database::models::module_hub::core::function::ModuleFunction;
use database::models::module_hub::detail::function::{
    ModuleFunctionDetail, NewModuleFunctionDetail,
};

use futures::future::join_all;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgQueryResult;
use sqlx::{query, query_as};

pub(crate) async fn get_functions_count(function_indexer_db: &PostgresPool) -> Count {
    query_as(
        "
                SELECT COUNT(id) as count
                FROM module_function
            ",
    )
    .fetch_one(function_indexer_db)
    .await
    .ok()
    .unwrap_or_default()
}

pub(crate) async fn get_function_detail(
    app_db: &PostgresPool,
    address: &str,
    module_name: &str,
    function_name: &str,
) -> Option<ModuleFunctionDetail> {
    query_as(
        "
                SELECT DISTINCT ON (address, module_name, function_name) *
                FROM module_function_detail
                WHERE
                    address = $1
                    AND
                    module_name = $2
                    AND
                    function_name = $3
                ORDER BY address, module_name, function_name, id DESC
            ",
    )
    .bind(address)
    .bind(module_name)
    .bind(function_name)
    .fetch_one(app_db)
    .await
    .ok()
}

pub async fn get_function_with_detail(
    function_db: &PostgresPool,
    app_db: &PostgresPool,
    address: &str,
    module_name: &str,
    function_name: &str,
) -> Option<ModuleFunctionWithDetail> {
    let maybe_function: Option<ModuleFunction> = query_as(
        "
                SELECT DISTINCT ON (module_address, module_name, name) *
                FROM module_function
                WHERE
                    module_address = $1
                    AND
                    module_name = $2
                    AND
                    name = $3
                ORDER BY module_address, module_name, name, id DESC
            ",
    )
    .bind(address)
    .bind(module_name)
    .bind(function_name)
    .fetch_one(function_db)
    .await
    .ok();

    match maybe_function {
        Some(function) => {
            let function_detail: Option<ModuleFunctionDetail> = query_as(
                "
                        SELECT DISTINCT ON (address, module_name, function_name) *
                        FROM module_function_detail
                        WHERE
                            address = $1
                            AND
                            module_name = $2
                            AND
                            function_name = $3
                        ORDER BY address, module_name, function_name, id DESC
                    ",
            )
            .bind(&function.module_address)
            .bind(&function.module_name)
            .bind(&function.name)
            .fetch_one(app_db)
            .await
            .ok();

            Some(ModuleFunctionWithDetail::compose(function, function_detail))
        }
        None => None,
    }
}

pub(crate) async fn get_functions_by_keyword_with_account_detail(
    function_indexer_db: &PostgresPool,
    app_db: &PostgresPool,
    keyword: &str,
) -> Option<Vec<ModuleFunctionWithAccountDetail>> {
    let address_by_alias = account::get_address_by_alias(app_db, keyword).await;

    let functions: Vec<ModuleFunction> = query_as(
        "
            SELECT DISTINCT ON (module_address, module_name, name) *
            FROM module_function
            WHERE
                module_address = $1
                OR
                module_address = $2
                OR
                module_name = $1
                OR
                name = $1
            ORDER BY module_address, module_name, name, id DESC
           ",
    )
    .bind(keyword)
    .bind(address_by_alias.unwrap_or_default())
    .fetch_all(function_indexer_db)
    .await
    .ok()
    .unwrap_or_default();

    let tasks = functions
        .into_iter()
        .map(|function| async {
            let account_detail =
                account::get_account_detail(app_db, &function.module_address).await;

            ModuleFunctionWithAccountDetail::compose(function, account_detail)
        })
        .collect::<Vec<_>>();

    let module_functions_with_account_detail: Vec<ModuleFunctionWithAccountDetail> =
        join_all(tasks).await;

    Some(module_functions_with_account_detail)
}

pub async fn get_paginated_functions_with_account_detail(
    function_indexer_db: &PostgresPool,
    app_db: &PostgresPool,
    offset: i64,
    limit: i64,
) -> Option<Vec<ModuleFunctionWithAccountDetail>> {
    let functions: Vec<ModuleFunction> = query_as(
        "
            SELECT DISTINCT ON (module_address, module_name, name) *
            FROM module_function
            ORDER BY module_address, module_name, name, id DESC
            OFFSET $1
            LIMIT $2
           ",
    )
    .bind(offset)
    .bind(limit)
    .fetch_all(function_indexer_db)
    .await
    .ok()
    .unwrap_or_default();

    let tasks = functions
        .into_iter()
        .map(|function| async {
            let account_detail =
                account::get_account_detail(app_db, &function.module_address).await;

            ModuleFunctionWithAccountDetail::compose(function, account_detail)
        })
        .collect::<Vec<_>>();

    let module_functions_with_account_detail: Vec<ModuleFunctionWithAccountDetail> =
        join_all(tasks).await;

    Some(module_functions_with_account_detail)
}

pub(crate) async fn get_functions_by_keyword_with_function_detail(
    function_indexer_db: &PostgresPool,
    app_db: &PostgresPool,
    keyword: &str,
) -> Result<Vec<ModuleFunctionWithOwnAndAccountDetail>, Error> {
    let address_by_alias = account::get_address_by_alias(app_db, keyword).await;

    let entry_functions: Vec<ModuleFunction> = query_as(
        "
            SELECT DISTINCT ON (module_address, module_name, name) *
            FROM module_function
            WHERE
                (
                    module_address = $1
                    OR
                    module_address = $2
                    OR
                    module_name = $1
                    OR
                    name = $1
                )
                AND
                is_entry = TRUE
            ORDER BY module_address, module_name, name, id DESC
           ",
    )
    .bind(keyword)
    .bind(address_by_alias.unwrap_or_default())
    .fetch_all(function_indexer_db)
    .await
    .ok()
    .unwrap_or_default();

    let tasks = entry_functions
        .into_iter()
        .map(|entry_function| async {
            let account_detail =
                account::get_account_detail(app_db, &entry_function.module_address).await;

            let function_detail = get_function_detail(
                app_db,
                &entry_function.module_address,
                &entry_function.module_name,
                &entry_function.name,
            )
            .await;

            ModuleFunctionWithOwnAndAccountDetail::compose(
                entry_function,
                account_detail,
                function_detail,
            )
        })
        .collect::<Vec<_>>();

    let module_functions_with_own_and_account_detail: Vec<ModuleFunctionWithOwnAndAccountDetail> =
        join_all(tasks).await;

    Ok(module_functions_with_own_and_account_detail)
}

pub(crate) async fn get_paginated_functions_by_keyword_with_function_detail(
    function_indexer_db: &PostgresPool,
    app_db: &PostgresPool,
    offset: i64,
    limit: i64,
) -> Result<Vec<ModuleFunctionWithOwnAndAccountDetail>, Error> {
    let entry_functions: Vec<ModuleFunction> = query_as(
        "
            SELECT DISTINCT ON (module_address, module_name, name) *
            FROM module_function
            WHERE is_entry = TRUE
            ORDER BY module_address, module_name, name, id DESC
            OFFSET $1
            LIMIT $2
           ",
    )
    .bind(offset)
    .bind(limit)
    .fetch_all(function_indexer_db)
    .await
    .ok()
    .unwrap_or_default();

    let tasks = entry_functions
        .into_iter()
        .map(|entry_function| async {
            let account_detail =
                account::get_account_detail(app_db, &entry_function.module_address).await;

            let function_detail = get_function_detail(
                app_db,
                &entry_function.module_address,
                &entry_function.module_name,
                &entry_function.name,
            )
            .await;

            ModuleFunctionWithOwnAndAccountDetail::compose(
                entry_function,
                account_detail,
                function_detail,
            )
        })
        .collect::<Vec<_>>();

    let module_functions_with_own_and_account_detail: Vec<ModuleFunctionWithOwnAndAccountDetail> =
        join_all(tasks).await;

    Ok(module_functions_with_own_and_account_detail)
}

pub async fn get_functions_by_address_and_module_name_with_detail(
    function_indexer_db: &PostgresPool,
    app_db: &PostgresPool,
    address: &str,
    module_name: &str,
) -> Option<Vec<ModuleFunctionWithDetail>> {
    let maybe_functions: Option<Vec<ModuleFunction>> = query_as(
        "
                SELECT DISTINCT ON (module_address, module_name, name) *
                FROM module_function
                WHERE
                    module_address = $1
                    AND
                    module_name = $2
                ORDER BY module_address, module_name, name, id DESC
            ",
    )
    .bind(address)
    .bind(module_name)
    .fetch_all(function_indexer_db)
    .await
    .ok();

    match maybe_functions {
        Some(functions) => {
            let tasks = functions
                .into_iter()
                .map(|function| async {
                    let function_detail = get_function_detail(
                        app_db,
                        &function.module_address,
                        &function.module_name,
                        &function.name,
                    )
                    .await;

                    ModuleFunctionWithDetail::compose(function, function_detail)
                })
                .collect::<Vec<_>>();

            let module_functions_with_detail: Vec<ModuleFunctionWithDetail> = join_all(tasks).await;

            Some(module_functions_with_detail)
        }
        None => None,
    }
}

pub(crate) async fn create_or_update_function_detail(
    function_indexer_db: &PostgresPool,
    app_db: &PostgresPool,
    function_detail: &NewModuleFunctionDetail,
) -> Result<PgQueryResult, Error> {
    #[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
    struct GenericTypeParams {
        constraints: Vec<String>,
    }

    let target_function: ModuleFunction = query_as(
        "
            SELECT DISTINCT ON (module_address, module_name, name) *
            FROM module_function
            WHERE
                module_address = $1
                AND
                module_name = $2
                AND
                name = $3
            ORDER BY module_address, module_name, name, id DESC
        ",
    )
    .bind(&function_detail.address)
    .bind(&function_detail.module_name)
    .bind(&function_detail.function_name)
    .fetch_one(function_indexer_db)
    .await
    .map_err(|_| Error::NotFound {
        msg: "onchain function not found".to_string(),
    })?;

    let target_function_params: Vec<String> =
        serde_json::from_value(target_function.params.clone().unwrap_or_default())
            .unwrap_or_default();

    let function_param_names: Vec<String> =
        serde_json::from_value(function_detail.param_names.clone().unwrap_or_default())
            .unwrap_or_default();

    let target_function_generic_param_names: Vec<GenericTypeParams> = serde_json::from_value(
        target_function
            .generic_type_params
            .clone()
            .unwrap_or_default(),
    )
    .unwrap_or_default();

    let generic_param_names: Vec<String> = serde_json::from_value(
        function_detail
            .generic_type_params
            .clone()
            .unwrap_or_default(),
    )
    .unwrap_or_default();

    if target_function_params.len() != function_param_names.len() {
        return Err(Error::InvalidParams {
            msg: "param_names length".to_string(),
        });
    }

    if target_function_generic_param_names.len() != generic_param_names.len() {
        return Err(Error::InvalidParams {
            msg: "generic_param_names length".to_string(),
        });
    }

    let maybe_function_detail: Option<ModuleFunctionDetail> = query_as(
        "
                SELECT *
                FROM module_function_detail
                WHERE
                    address = $1
                    AND
                    module_name = $2
                    AND
                    function_name = $3
                ORDER BY id DESC
            ",
    )
    .bind(&function_detail.address)
    .bind(&function_detail.module_name)
    .bind(&function_detail.function_name)
    .fetch_one(app_db)
    .await
    .ok();

    let query = match maybe_function_detail {
        Some(current_function_detail) => query(
            "
                    UPDATE module_function_detail SET
                        description = $1,
                        param_names = $2,
                        generic_type_params = $3
                    WHERE id = $4
                ",
        )
        .bind(&function_detail.description)
        .bind(&function_detail.param_names)
        .bind(&function_detail.generic_type_params)
        .bind(current_function_detail.id),
        None => query(
            "
                    INSERT INTO module_function_detail
                        (address, module_name, function_name, description, param_names, generic_type_params)
                        VALUES
                        ($1, $2, $3, $4, $5, $6)
                ",
        )
        .bind(&function_detail.address)
        .bind(&function_detail.module_name)
        .bind(&function_detail.function_name)
        .bind(&function_detail.description)
        .bind(&function_detail.param_names)
        .bind(&function_detail.generic_type_params),
    };

    query
        .execute(app_db)
        .await
        .map_err(|e| Error::DbError(e.into()))
}
