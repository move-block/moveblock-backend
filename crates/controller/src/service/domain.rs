pub mod account;
pub mod block_stack;
pub mod function;
pub mod module;

use crate::service::Error;
use aptos_sdk::crypto::ed25519::{Ed25519PublicKey, Ed25519Signature};
use aptos_sdk::crypto::{Signature, ValidCryptoMaterialStringExt};
use std::ops::Add;
use std::str::FromStr;

use aptos_sdk::types::account_address::{create_resource_address, AccountAddress};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, Deserialize)]
pub struct Response<T>
where
    T: Serialize,
{
    data: T,
    pagination: Pagination,
}

#[derive(Serialize, Deserialize)]
pub struct Pagination {
    /// Only Some when queried with pagination params
    pub offset: Option<i64>,
    /// Only Some when queried with pagination params
    pub limit: Option<i64>,
    pub total_len: i64,
}

#[derive(Serialize, Deserialize, Default, Debug, FromRow)]
pub struct Count {
    pub count: i64,
}

impl<T> Response<T>
where
    T: Serialize,
{
    pub fn new(total_len: i64, data: T, offset: Option<i64>, limit: Option<i64>) -> Self {
        Response {
            data,
            pagination: Pagination {
                offset,
                limit,
                total_len,
            },
        }
    }
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
pub struct SignedBody {
    pub payload: SignedData,
    pub public_key: String,
}

pub fn get_resource_account(source: &str) -> Result<String, Error> {
    Ok(String::from("0x").add(
        create_resource_address(
            AccountAddress::from_str(source).map_err(|_| Error::InvalidParams {
                msg: "cannot parse address".to_string(),
            })?,
            &[0],
        )
        .to_string()
        .as_str(),
    ))
}

pub async fn verify<T>(body: &SignedBody, address: &str) -> Result<T, Error>
where
    T: Serialize + DeserializeOwned + std::fmt::Debug,
{
    let signature = Ed25519Signature::from_encoded_string(&body.payload.signature)
        .map_err(anyhow::Error::new)?;
    let public_key =
        Ed25519PublicKey::from_encoded_string(&body.public_key).map_err(anyhow::Error::new)?;

    if !body.payload.full_message.contains(&body.payload.message) {
        return Err(Error::NotFound {
            msg: "payload message is cannot be parsed from fullMessage".to_string(),
        });
    }

    signature.verify_arbitrary_msg(body.payload.full_message.as_bytes(), &public_key)?;

    let resource_account = get_resource_account(&body.payload.address)?;

    if body.payload.address != address && resource_account != address {
        return Err(Error::UnAuthorized {});
    }

    Ok(serde_json::from_str::<T>(&body.payload.message).map_err(anyhow::Error::new)?)
}

#[cfg(test)]
mod signature {
    use crate::service::domain::{verify, SignedBody};
    use serde::{Deserialize, Serialize};
    use serde_json::from_str;

    #[test]
    fn test_sign() {
        test_sign_inner();
    }

    #[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
    struct ModuleDetailBody {
        pub description: Option<String>,
        pub github_url: Option<String>,
        pub r#type: Option<String>,
    }

    #[actix_rt::test]
    async fn test_sign_inner() {
        let msg = r#"
        {
            "data": {
                "address": "0x44ed46e3943de0ec15ea8edf0a40aa111dc82f8cfcdd82a712ad1352079f21b2",
                "application": "https://petra.app",
                "chainId": 42,
                "fullMessage": "APTOS\nmessage: {\"description\":\"description of any module\",\"github_url\":\"github.com\",\"type\":\"NFT\"}\nnonce: test",
                "message": "{\"description\":\"description of any module\",\"github_url\":\"github.com\",\"type\":\"NFT\"}",
                "nonce": "test",
                "prefix": "APTOS",
                "signature": "3751ac64dc7170223da4e98005f0c4178590dedf6d26d788b79ac181576212da9487015f6e15fc0514b17c6bee5c94d9a13be0b3baf3bbbaeb37a05ab5612002"
            },
            "public_key": "0x4f96cac93f78add3aabff81f6bbb5d1d9a0f7aa7fffbd2e13c3acf9c35f69bda"
        }
        "#;

        let signed_body: SignedBody = from_str(msg).unwrap();

        let module_detail = verify::<ModuleDetailBody>(
            &signed_body,
            "0x4f96cac93f78add3aabff81f6bbb5d1d9a0f7aa7fffbd2e13c3acf9c35f69bda",
        )
        .await
        .unwrap();

        assert_eq!(
            module_detail,
            ModuleDetailBody {
                description: Some("description of any module".to_string()),
                github_url: Some("github.com".to_string()),
                r#type: Some("NFT".to_string()),
            }
        )
    }
}
