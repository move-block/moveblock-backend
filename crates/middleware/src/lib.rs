use actix_utils::future::{ready, Ready};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::StatusCode;
use actix_web::Error;
// use aptos_crypto::ed25519::{Ed25519PublicKey, Ed25519Signature};
// use aptos_crypto::{Signature, ValidCryptoMaterialStringExt};
use futures_util::future::LocalBoxFuture;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedBody {
    pub payload: SignedData,
    pub public_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignedData {
    pub address: String,
    pub application: String,
    #[serde(rename = "chainId")]
    pub chain_id: i64,
    #[serde(rename = "fullMessage")]
    pub full_message: String,
    pub message: String,
    pub prefix: String,
    pub signature: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Auth {
    pub address: String,
    pub timestamp: String,
}

impl Auth {
    pub fn new(address: String, timestamp: String) -> Self {
        Auth { address, timestamp }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MiddlewareError {
    #[error("UnAuthorized")]
    UnAuthorized {},

    #[error("BadRequest")]
    BadRequest {},
}

impl actix_web::ResponseError for MiddlewareError {
    fn status_code(&self) -> StatusCode {
        match self {
            MiddlewareError::UnAuthorized {} => StatusCode::UNAUTHORIZED,
            MiddlewareError::BadRequest {} => StatusCode::BAD_REQUEST,
        }
    }
}

pub struct AptosAuth;

impl AptosAuth {
    pub fn build() -> Self {
        Self
    }
}

impl<S, B> Transform<S, ServiceRequest> for AptosAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AptosAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AptosAuthMiddleware { service }))
    }
}

pub struct AptosAuthMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AptosAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    actix_service::forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let signature = req.headers().get("aptos-auth");

        let signature = signature
            .map(|s| s.to_str().unwrap_or_default().to_string())
            .unwrap_or_default();

        // let public_key = public_key
        //     .map(|p| p.to_str().unwrap_or_default().to_string())
        //     .unwrap_or_default();
        // let timestamp = timestamp
        //     .map(|t| t.to_str().unwrap_or_default().to_string())
        //     .unwrap_or_default();

        let address = req
            .path()
            .strip_prefix("/api/v1/block-stacks/")
            .unwrap_or_default()
            .split('/')
            .take(1)
            .collect::<String>();

        if signature != address {
            return Box::pin(async move { Err(Error::from(MiddlewareError::UnAuthorized {})) });
        }

        let fut = self.service.call(req);

        Box::pin(async move {
            let next_service = fut.await?;

            // let signature = Ed25519Signature::from_encoded_string(&signature).map_err(|_| Error::from(MiddlewareError::UnAuthorized {}))?;
            // let public_key = Ed25519PublicKey::from_encoded_string(&public_key).map_err(|_| Error::from(MiddlewareError::UnAuthorized {}))?;

            // let auth = Auth::new(address, timestamp);
            // let auth = serde_json::to_string(&auth).map_err(|_| Error::from(MiddlewareError::BadRequest {}))?;

            Ok(next_service)
        })
    }
}
