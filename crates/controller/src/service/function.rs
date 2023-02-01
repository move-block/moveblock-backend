use crate::config::ApiContext;
use crate::service::{
    domain::{function, Response},
    Error,
};

use actix_web::{
    get,
    web::{self, Data, Query},
    Responder,
};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct FunctionQueryParams {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
    pub keyword: Option<String>,
}

#[get("")]
async fn functions_by_params(
    context: Data<ApiContext>,
    params: Query<FunctionQueryParams>,
) -> Result<impl Responder, Error> {
    let (total_len, functions, offset, limit) = match &params.keyword {
        Some(keyword) => {
            let functions = function::get_functions_by_keyword_with_account_detail(
                &context.function_index_db,
                &context.app_db,
                keyword,
            )
            .await
            .unwrap_or_default();

            (functions.len() as i64, functions, None, None)
        }
        None => {
            let offset = params.offset.unwrap_or(0);
            let limit = params.limit.unwrap_or(10);
            let functions = function::get_paginated_functions_with_account_detail(
                &context.function_index_db,
                &context.app_db,
                offset,
                limit,
            )
            .await
            .unwrap_or_default();

            let count = function::get_functions_count(&context.function_index_db).await;

            (count.count, functions, Some(offset), Some(limit))
        }
    };

    Ok(web::Json(Response::new(
        total_len, functions, offset, limit,
    )))
}

#[get("/{address}/{module_name}/{function_name}")]
async fn function_detail(
    context: Data<ApiContext>,
    params: web::Path<(String, String, String)>,
) -> Result<impl Responder, Error> {
    let (address, module_name, function_name) = params.into_inner();
    let function_detail =
        function::get_function_detail(&context.app_db, &address, &module_name, &function_name)
            .await;
    Ok(web::Json(function_detail))
}

#[get("/entry-functions")]
async fn entry_functions_by_params(
    context: Data<ApiContext>,
    params: Query<FunctionQueryParams>,
) -> Result<impl Responder, Error> {
    let (total_len, functions, offset, limit) = match &params.keyword {
        Some(keyword) => {
            let functions = function::get_functions_by_keyword_with_function_detail(
                &context.function_index_db,
                &context.app_db,
                keyword,
            )
            .await
            .unwrap_or_default();

            (functions.len() as i64, functions, None, None)
        }
        None => {
            let offset = params.offset.unwrap_or(0);
            let limit = params.limit.unwrap_or(10);
            let functions = function::get_paginated_functions_by_keyword_with_function_detail(
                &context.function_index_db,
                &context.app_db,
                offset,
                limit,
            )
            .await
            .unwrap_or_default();

            let count = function::get_functions_count(&context.function_index_db).await;

            (count.count, functions, Some(offset), Some(limit))
        }
    };

    Ok(web::Json(Response::new(
        total_len, functions, offset, limit,
    )))
}

pub fn routers(scope: actix_web::Scope) -> actix_web::Scope {
    scope
        .service(functions_by_params)
        .service(entry_functions_by_params)
        .service(function_detail)
}
