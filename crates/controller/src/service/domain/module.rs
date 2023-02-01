use crate::service::{
    domain::{account, function, Count},
    Error,
};

use database::db::PostgresPool;
use database::models::module_hub::composite::module::{
    MoveModuleWithDetail, MoveModuleWithFunctionsAccountAndDetails,
};
use database::models::module_hub::core::module::MoveModule;
use database::models::module_hub::detail::module::{ModuleDetail, NewModuleDetail};

use sqlx::postgres::PgQueryResult;
use sqlx::{query, query_as};

pub(crate) async fn get_modules_by_account(
    aptos_full_index_db: &PostgresPool,
    app_db: &PostgresPool,
    address_or_alias: &str,
) -> Option<Vec<MoveModule>> {
    let address_by_alias = account::get_address_by_alias(app_db, address_or_alias).await;

    query_as(
        "
                SELECT DISTINCT ON (name, address) *
                FROM move_modules
                WHERE
                    address = $1
                    OR
                    address = $2
                ORDER BY name, address, transaction_version DESC
            ",
    )
    .bind(address_or_alias)
    .bind(address_by_alias.unwrap_or_default())
    .fetch_all(aptos_full_index_db)
    .await
    .ok()
}

pub(crate) async fn get_module_by_address_name_with_detail(
    aptos_full_index_db: &PostgresPool,
    app_db: &PostgresPool,
    address: &str,
    module_name: &str,
) -> Option<MoveModuleWithDetail> {
    let maybe_module: Option<MoveModule> = query_as(
        "
                SELECT DISTINCT ON (name, address) *
                FROM move_modules
                WHERE
                    address = $1
                    AND
                    name = $2
                ORDER BY name, address, transaction_version DESC
            ",
    )
    .bind(address.to_string())
    .bind(module_name.to_string())
    .fetch_one(aptos_full_index_db)
    .await
    .ok();

    match maybe_module {
        Some(module) => {
            let module_detail: Option<ModuleDetail> = query_as(
                "
                        SELECT DISTINCT ON (address, module_name) *
                        FROM module_detail
                        WHERE
                            address = $1
                            AND
                            module_name = $2
                        ORDER BY address, module_name, id DESC
                    ",
            )
            .bind(&module.address)
            .bind(&module.name)
            .fetch_one(app_db)
            .await
            .ok();

            Some(MoveModuleWithDetail::compose(module, module_detail))
        }
        None => None,
    }
}

pub(crate) async fn get_module_with_functions_and_details(
    aptos_full_index_db: &PostgresPool,
    function_index_db: &PostgresPool,
    app_db: &PostgresPool,
    address: &str,
    module_name: &str,
) -> Option<MoveModuleWithFunctionsAccountAndDetails> {
    let maybe_move_module_with_detail =
        get_module_by_address_name_with_detail(aptos_full_index_db, app_db, address, module_name)
            .await;

    match maybe_move_module_with_detail {
        Some(move_module_with_detail) => {
            let maybe_functions = function::get_functions_by_address_and_module_name_with_detail(
                function_index_db,
                app_db,
                address,
                module_name,
            )
            .await
            .unwrap_or_default();

            let account_detail = account::get_account_detail(app_db, address).await;
            Some(MoveModuleWithFunctionsAccountAndDetails::compose(
                move_module_with_detail,
                account_detail,
                maybe_functions,
            ))
        }
        None => None,
    }
}

pub(crate) async fn create_or_update_module_info(
    aptos_full_index_db: &PostgresPool,
    app_db: &PostgresPool,
    module_detail: &NewModuleDetail,
) -> Result<PgQueryResult, Error> {
    let target_module_count: Count = query_as(
        "
                SELECT COUNT(name) as count
                FROM move_modules
                WHERE
                    address = $1
                    AND
                    name = $2
            ",
    )
    .bind(&module_detail.address)
    .bind(&module_detail.module_name)
    .fetch_one(aptos_full_index_db)
    .await
    .unwrap_or_default();

    if target_module_count.count == 0 {
        return Err(Error::NotFound {
            msg: "onchain module not found".to_string(),
        });
    }

    let maybe_module_detail: Option<ModuleDetail> = query_as(
        "
                SELECT *
                FROM module_detail
                WHERE
                    address = $1
                    AND
                    module_name = $2
                ORDER BY id DESC
            ",
    )
    .bind(&module_detail.address)
    .bind(&module_detail.module_name)
    .fetch_one(app_db)
    .await
    .ok();

    let query = match maybe_module_detail {
        Some(current_module) => query(
            "
                        UPDATE module_detail SET
                            description = $1,
                            github_url = $2,
                            rev = $3,
                            subdir = $4
                        WHERE id = $5
                    ",
        )
        .bind(&module_detail.description)
        .bind(&module_detail.github_url)
        .bind(&module_detail.rev)
        .bind(&module_detail.subdir)
        .bind(current_module.id),
        None => query(
            "
                        INSERT INTO module_detail
                            (address, module_name, description, github_url, rev, subdir)
                            VALUES
                            ($1, $2, $3, $4, $5, $6)
                    ",
        )
        .bind(&module_detail.address)
        .bind(&module_detail.module_name)
        .bind(&module_detail.description)
        .bind(&module_detail.github_url)
        .bind(&module_detail.rev)
        .bind(&module_detail.subdir),
    };

    query
        .execute(app_db)
        .await
        .map_err(|e| Error::DbError(e.into()))
}

pub async fn get_module_detail(
    app_db: &PostgresPool,
    address: &str,
    module_name: &str,
) -> Option<ModuleDetail> {
    query_as(
        "
                SELECT DISTINCT ON (address, module_name) *
                FROM module_detail
                WHERE
                    address = $1
                    AND
                    module_name = $2
                ORDER BY address, module_name, id DESC
            ",
    )
    .bind(address)
    .bind(module_name)
    .fetch_one(app_db)
    .await
    .ok()
}
