use crate::service::block_stack::Block;
use crate::service::domain::{module, Count};
use crate::service::Error;

use database::db::PostgresPool;
use database::models::block_stack::stack::{BlockStack, NewBlockStack};
use move_generator::{Dependency, Function, MoveScript};

use sqlx::postgres::PgQueryResult;
use sqlx::types::JsonValue;
use sqlx::{query, query_as};

pub async fn get_my_block_stacks_count(app_db: &PostgresPool, address: &str) -> Count {
    query_as(
        "
                SELECT COUNT(id) as count
                FROM block_stack
                WHERE address = $1
            ",
    )
    .bind(address)
    .fetch_one(app_db)
    .await
    .ok()
    .unwrap_or_default()
}

pub(crate) async fn get_my_block_stacks(
    app_db: &PostgresPool,
    address: &str,
    offset: i64,
    limit: i64,
) -> Option<Vec<BlockStack>> {
    query_as(
        "
                SELECT *
                FROM block_stack
                WHERE address = $1
                ORDER BY id ASC
                OFFSET $2
                LIMIT $3
            ",
    )
    .bind(address)
    .bind(offset)
    .bind(limit)
    .fetch_all(app_db)
    .await
    .ok()
}

pub(crate) async fn get_block_stack(app_db: &PostgresPool, id: i32) -> Option<BlockStack> {
    query_as(
        "
                SELECT *
                FROM block_stack
                WHERE id = $1
            ",
    )
    .bind(id)
    .fetch_one(app_db)
    .await
    .ok()
}

pub(crate) async fn get_script_bytecode(app_db: &PostgresPool, id: i32) -> Option<String> {
    let maybe_block_stack: Option<BlockStack> = query_as(
        "
                SELECT *
                FROM block_stack
                WHERE id = $1
            ",
    )
    .bind(id)
    .fetch_one(app_db)
    .await
    .ok();

    if let Some(block_stack) = maybe_block_stack {
        Some(String::from_utf8_lossy(block_stack.bytecode.as_slice()).to_string())
    } else {
        None
    }
}
/// Auth-checked address at upper layer
pub(crate) async fn create_my_block_stack(
    app_db: &PostgresPool,
    address: String,
    new_block_stack: NewBlockStack,
) -> Result<(), Error> {
    let db = app_db.clone();
    let stack = new_block_stack.stack.clone();
    let account = address.clone();

    let res = query(
        "
                INSERT INTO block_stack
                (address, name, stack) VALUES ($1, $2, $3)
             ",
    )
    .bind(&address)
    .bind(&new_block_stack.name)
    .bind(&new_block_stack.stack)
    .execute(app_db)
    .await
    .map_err(|e| Error::DbError(e.into()));

    if res.is_ok() {
        actix_rt::task::spawn_blocking(move || {
            futures::executor::block_on(create_bytecode(db, stack, account)).unwrap_or_default();
        });
    }

    Ok(())
}

async fn create_bytecode(
    app_db: PostgresPool,
    stack: JsonValue,
    address: String,
) -> Result<(), Error> {
    async fn parse_deps_and_functions(
        app_db: &PostgresPool,
        stack: &JsonValue,
    ) -> (Vec<Dependency>, Vec<Function>) {
        let mut dependencies = Vec::new();
        let mut functions = Vec::new();

        let stack: Vec<Block> =
            serde_json::from_value::<Vec<Block>>(stack.clone()).unwrap_or_default();

        for block in stack {
            let split_function = block.function.split("::").collect::<Vec<_>>();
            let (address, module_name, _) =
                (split_function[0], split_function[1], split_function[2]);
            let maybe_module_detail = module::get_module_detail(app_db, address, module_name).await;
            if let Some(module_detail) = maybe_module_detail {
                let dependency = Dependency::new(
                    &module_detail.github_url.unwrap_or_default(),
                    &module_detail.rev.unwrap_or_default(),
                    &module_detail.subdir.unwrap_or_default(),
                );

                let function =
                    Function::new(&block.function, block.type_arguments, block.arguments);

                if !dependencies.contains(&dependency) {
                    dependencies.push(dependency);
                }

                functions.push(function);
            }
        }

        (dependencies, functions)
    }

    let (dependencies, functions) = parse_deps_and_functions(&app_db, &stack).await;
    if functions.is_empty() {
        return Err(Error::NotFound {
            msg: "module detail missing".to_string(),
        });
    }

    let mut move_script = MoveScript::new()
        .init()
        .add_dependencies(dependencies)
        .add_functions(functions);

    let compile_res = move_script
        .generate_script()
        .await
        .map_err(|e| Error::AnyError(anyhow::Error::new(e)))?;

    let path = compile_res
        .dir
        .join("build")
        .join("block-stack")
        .join("bytecode_scripts")
        .join("main.mv");

    match tokio::fs::read(path).await {
        Ok(compiled_script) => {
            println!(
                "{}",
                String::from_utf8_lossy(compile_res.output.stdout.as_slice())
            );

            let hex_encoded_script = hex::encode(compiled_script);
            query(
                "
                UPDATE block_stack SET bytecode = $1
                WHERE address = $2
            ",
            )
            .bind(hex_encoded_script.as_bytes())
            .bind(address)
            .execute(&app_db)
            .await
            .ok();
        }
        Err(_) => {
            println!(
                "{}",
                String::from_utf8_lossy(compile_res.output.stderr.as_slice())
            );
        }
    }
    move_script.destroy_self().await.unwrap_or_default();
    Ok(())
}

pub(crate) async fn update_my_block_stack(
    app_db: &PostgresPool,
    address: String,
    id: i32,
    new_block_stack: NewBlockStack,
) -> Result<(), Error> {
    let db = app_db.clone();
    let stack = new_block_stack.stack.clone();
    let account = address.clone();

    let target_block_stack: BlockStack = query_as(
        "
                SELECT *
                FROM block_stack
                WHERE id = $1
            ",
    )
    .bind(id)
    .fetch_one(app_db)
    .await
    .map_err(|_| Error::NotFound {
        msg: "block stack not found".to_string(),
    })?;

    if id != target_block_stack.id {
        return Err(Error::UnAuthorized {});
    }

    if address != target_block_stack.address {
        return Err(Error::UnAuthorized {});
    }

    let res = query(
        "
                UPDATE block_stack
                    SET
                        stack = $1,
                        name = $2,
                        last_edit_datetime = now()
                WHERE id = $3
            ",
    )
    .bind(&new_block_stack.stack)
    .bind(&new_block_stack.name)
    .bind(id)
    .execute(app_db)
    .await
    .map_err(|e| Error::DbError(e.into()));

    if res.is_ok() {
        actix_rt::task::spawn_blocking(move || {
            futures::executor::block_on(create_bytecode(db, stack, account)).unwrap_or_default();
        });
    }

    Ok(())
}

pub(crate) async fn delete_my_block_stack(
    app_db: &PostgresPool,
    address: String,
    id: i32,
) -> Result<PgQueryResult, Error> {
    let target_block_stack: BlockStack = query_as(
        "
                SELECT *
                FROM block_stack
                WHERE id = $1
            ",
    )
    .bind(id)
    .fetch_one(app_db)
    .await
    .map_err(|_| Error::NotFound {
        msg: "block stack not found".to_string(),
    })?;

    if address != target_block_stack.address {
        return Err(Error::UnAuthorized {});
    }

    if id != target_block_stack.id {
        return Err(Error::UnAuthorized {});
    }

    query(
        "
                DELETE
                FROM block_stack
                WHERE id = $1
            ",
    )
    .bind(id)
    .execute(app_db)
    .await
    .map_err(|e| Error::DbError(e.into()))
}
