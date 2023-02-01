use crate::service::{domain::Count, Error};

use database::db::PostgresPool;
use database::models::module_hub::detail::account::AccountDetail;

use sqlx::postgres::PgQueryResult;
use sqlx::{query, query_as};

pub(crate) async fn get_address_by_alias(app_db: &PostgresPool, alias: &str) -> Option<String> {
    let maybe_address_by_alias: Option<AccountDetail> =
        query_as("SELECT * FROM account_detail WHERE alias = $1")
            .bind(alias)
            .fetch_one(app_db)
            .await
            .ok();

    match maybe_address_by_alias {
        Some(account_detail) => Some(account_detail.address),
        None => None,
    }
}

pub(crate) async fn get_account_detail(
    app_db: &PostgresPool,
    address: &str,
) -> Option<AccountDetail> {
    query_as("SELECT * FROM account_detail WHERE address = $1")
        .bind(address)
        .fetch_one(app_db)
        .await
        .ok()
}

pub(crate) async fn create_or_update_account_alias(
    aptos_full_index_db: &PostgresPool,
    app_db: &PostgresPool,
    address: &str,
    alias: &Option<String>,
) -> Result<PgQueryResult, Error> {
    let account_count: Count = query_as(
        "
            SELECT COUNT(address) as count
            FROM move_modules
            WHERE address = $1
        ",
    )
    .bind(address)
    .fetch_one(aptos_full_index_db)
    .await
    .unwrap_or_default();

    if account_count.count == 0 {
        return Err(Error::NotFound {
            msg: "on-chain account not found".to_string(),
        });
    }

    let maybe_account_detail: Option<AccountDetail> = query_as(
        "
                SELECT *
                FROM account_detail
                WHERE address = $1
                ORDER BY id DESC
            ",
    )
    .bind(address)
    .fetch_one(app_db)
    .await
    .ok();

    let query = match maybe_account_detail {
        Some(account) => query(
            "
                    UPDATE account_detail SET alias = $1
                    WHERE id = $2
                ",
        )
        .bind(alias)
        .bind(account.id),
        None => query(
            "
                    INSERT INTO account_detail (address, alias) VALUES ($1, $2)
                ",
        )
        .bind(address)
        .bind(alias),
    };

    query
        .execute(app_db)
        .await
        .map_err(|e| Error::DbError(e.into()))
}
