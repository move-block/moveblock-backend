use crate::config::ApiContext;
use crate::service::{
    domain::{block_stack, Response},
    Error,
};

use actix_web::{
    delete, get, patch, post,
    web::{self, Data, Query},
    HttpResponse, Responder,
};

use database::models::block_stack::stack::NewBlockStack;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct BlockStackQueryParams {
    pub offset: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Block {
    pub r#type: String,
    pub function: String,
    pub type_arguments: Vec<String>,
    pub arguments: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockStackBody {
    pub name: String,
    pub blocks: Vec<Block>,
}

impl TryFrom<BlockStackBody> for NewBlockStack {
    type Error = Error;

    fn try_from(value: BlockStackBody) -> Result<Self, Self::Error> {
        Ok(NewBlockStack {
            name: value.name,
            stack: serde_json::to_value(value.blocks).map_err(|_| Error::InvalidParams {
                msg: "cannot parse stack".to_string(),
            })?,
        })
    }
}

#[get("/{address}")]
async fn block_stacks(
    context: Data<ApiContext>,
    address: web::Path<String>,
    params: Query<BlockStackQueryParams>,
) -> Result<impl Responder, Error> {
    let address = address.into_inner();
    let offset = params.offset.unwrap_or(0);
    let limit = params.limit.unwrap_or(10);

    let block_stacks =
        block_stack::get_my_block_stacks(&context.app_db, &address, offset, limit).await;

    let count = block_stack::get_my_block_stacks_count(&context.app_db, &address).await;

    Ok(web::Json(Response::new(
        count.count,
        block_stacks,
        Some(offset),
        Some(limit),
    )))
}

#[post("/{address}")]
async fn create_block_stack(
    context: Data<ApiContext>,
    address: web::Path<String>,
    body: web::Json<BlockStackBody>,
) -> Result<impl Responder, Error> {
    let address = address.into_inner();
    block_stack::create_my_block_stack(&context.app_db, address, body.into_inner().try_into()?)
        .await?;

    Ok(HttpResponse::Ok())
}

#[get("/{address}/stacks/{id}")]
async fn get_block_stack(
    context: Data<ApiContext>,
    path: web::Path<(String, i32)>,
) -> Result<impl Responder, Error> {
    let (_, id) = path.into_inner();
    let bs = block_stack::get_block_stack(&context.app_db, id).await;
    Ok(web::Json(bs))
}

#[get("/{address}/stacks/{id}/atomic")]
async fn get_script_bytecode(
    context: Data<ApiContext>,
    path: web::Path<(String, i32)>,
) -> Result<impl Responder, Error> {
    let (_, id) = path.into_inner();
    let bs = block_stack::get_script_bytecode(&context.app_db, id).await;
    Ok(web::Json(bs))
}

#[patch("/{address}/stacks/{id}")]
async fn update_block_stack(
    context: Data<ApiContext>,
    path: web::Path<(String, i32)>,
    body: web::Json<BlockStackBody>,
) -> Result<impl Responder, Error> {
    let (address, id) = path.into_inner();
    block_stack::update_my_block_stack(&context.app_db, address, id, body.into_inner().try_into()?)
        .await?;
    Ok(HttpResponse::Ok())
}

#[delete("/{address}/stacks/{id}")]
async fn delete_block_stack(
    context: Data<ApiContext>,
    path: web::Path<(String, i32)>,
) -> Result<impl Responder, Error> {
    let (address, id) = path.into_inner();
    block_stack::delete_my_block_stack(&context.app_db, address, id).await?;
    Ok(HttpResponse::Ok())
}

#[get("/{address}/execute/stacks/{id}")]
async fn execute_script(
    context: Data<ApiContext>,
    path: web::Path<(String, i32)>,
) -> Result<impl Responder, Error> {
    let (address, id) = path.into_inner();
    let res = block_stack::execute_script(&context.app_db, address, id).await?;

    Ok(web::Json(res))
}

pub fn routers(scope: actix_web::Scope) -> actix_web::Scope {
    scope
        .service(block_stacks)
        .service(create_block_stack)
        .service(update_block_stack)
        .service(delete_block_stack)
        .service(get_block_stack)
        .service(get_script_bytecode)
        .service(execute_script)
}
