use crate::config::ApiContext;
use crate::service::domain::{self, Response, SignedBody};
use crate::service::Error;

use database::models::module_hub::detail::function::NewModuleFunctionDetail;
use database::models::module_hub::detail::module::NewModuleDetail;

use actix_web::{
    get, post,
    web::{self, Data},
    HttpResponse, Responder,
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct PathParams {
    pub address_or_alias: String,
    pub module_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct AccountAliasBody {
    pub alias: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct ModuleDetailBody {
    pub description: Option<String>,
    pub github_url: Option<String>,
    pub rev: Option<String>,
    pub subdir: Option<String>,
    pub alias: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ModulePostParams {
    pub address: String,
    pub module_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct FunctionPostParams {
    pub address: String,
    pub module_name: String,
    pub function_name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct FunctionDetailBody {
    pub description: Option<String>,
    pub param_names: Option<Vec<String>>,
    pub generic_type_params: Option<Vec<String>>,
}

#[get("/{address}/contains/{module_address}")]
async fn check_module_auth(path: web::Path<(String, String)>) -> Result<impl Responder, Error> {
    let mut result = false;

    let (address, module_address) = path.into_inner();
    let resource_account = domain::get_resource_account(&address)?;

    if address == module_address || resource_account == module_address {
        result = true;
    }

    Ok(web::Json(result))
}

#[get("/{address_or_alias}/modules")]
async fn modules_by_address_or_alias(
    path: web::Path<PathParams>,
    context: Data<ApiContext>,
) -> Result<impl Responder, Error> {
    let modules = domain::module::get_modules_by_account(
        &context.aptos_full_index_db,
        &context.app_db,
        &path.address_or_alias,
    )
    .await
    .unwrap_or_default();

    Ok(web::Json(Response::new(
        modules.len() as i64,
        modules,
        None,
        None,
    )))
}

#[get("/{address_or_alias}/modules/{module_name}")]
async fn functions_by_account_and_module_name(
    path: web::Path<PathParams>,
    context: Data<ApiContext>,
) -> Result<impl Responder, Error> {
    let modules = domain::module::get_module_with_functions_and_details(
        &context.aptos_full_index_db,
        &context.function_index_db,
        &context.app_db,
        &path.address_or_alias,
        &path.module_name.clone().unwrap_or_default(),
    )
    .await;

    let total_len = match modules.clone() {
        Some(modules) => modules.functions_with_detail.len() as i64,
        None => 0,
    };

    Ok(web::Json(Response::new(total_len, modules, None, None)))
}

#[post("/{address}")]
async fn account_alias(
    context: Data<ApiContext>,
    path: web::Path<String>,
    body: web::Json<SignedBody>,
) -> Result<impl Responder, Error> {
    let verified_body: AccountAliasBody = domain::verify(&body.into_inner(), path.as_str()).await?;

    domain::account::create_or_update_account_alias(
        &context.aptos_full_index_db,
        &context.app_db,
        path.as_str(),
        &Some(verified_body.alias),
    )
    .await?;

    Ok(HttpResponse::Ok())
}

#[post("/{address}/modules/{module_name}")]
async fn module_detail(
    context: Data<ApiContext>,
    path: web::Path<ModulePostParams>,
    body: web::Json<SignedBody>,
) -> Result<impl Responder, Error> {
    let verified_body: ModuleDetailBody = domain::verify(&body.into_inner(), &path.address).await?;

    let new_module_detail = NewModuleDetail::new(
        &path.address,
        &path.module_name,
        &verified_body.description,
        &verified_body.github_url,
        &verified_body.rev,
        &verified_body.subdir,
    );

    domain::account::create_or_update_account_alias(
        &context.aptos_full_index_db,
        &context.app_db,
        &path.address,
        &verified_body.alias,
    )
    .await?;

    domain::module::create_or_update_module_info(
        &context.aptos_full_index_db,
        &context.app_db,
        &new_module_detail,
    )
    .await?;

    Ok(HttpResponse::Ok())
}

#[post("/{address}/modules/{module_name}/functions/{function_name}")]
async fn function_detail(
    context: Data<ApiContext>,
    path: web::Path<FunctionPostParams>,
    body: web::Json<SignedBody>,
) -> Result<impl Responder, Error> {
    let verified_body: FunctionDetailBody =
        domain::verify(&body.into_inner(), &path.address).await?;

    let new_function_detail = NewModuleFunctionDetail::new(
        &path.address,
        &path.module_name,
        &path.function_name,
        verified_body.description,
        verified_body.param_names,
        verified_body.generic_type_params,
    );

    domain::function::create_or_update_function_detail(
        &context.function_index_db,
        &context.app_db,
        &new_function_detail,
    )
    .await?;

    Ok(HttpResponse::Ok())
}

pub fn routers(scope: actix_web::Scope) -> actix_web::Scope {
    scope
        .service(check_module_auth)
        .service(modules_by_address_or_alias)
        .service(functions_by_account_and_module_name)
        .service(account_alias)
        .service(module_detail)
        .service(function_detail)
}
