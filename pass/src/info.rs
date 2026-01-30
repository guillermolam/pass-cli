use crate::PassClient;
use anyhow::{Context, Result};
use muon::GET;
use muon::env::EnvId;
use pass_domain::crypto;

#[derive(Debug)]
pub struct UserInfo {
    pub user: UserInfoUser,
    pub env: EnvId,
}

#[derive(Debug)]
pub struct UserInfoUser {
    pub id: String,
    pub name: String,
    pub email: String,
}

impl From<UserResponse> for UserInfoUser {
    fn from(value: UserResponse) -> Self {
        Self {
            id: value.id,
            name: value.name.unwrap_or_else(|| value.email.clone()),
            email: value.email,
        }
    }
}

#[derive(Debug, serde::Deserialize)]
struct GetUserResponse {
    #[serde(rename = "User")]
    user: UserResponse,
}

#[derive(Debug, serde::Deserialize)]
struct UserResponse {
    #[serde(rename = "ID")]
    pub id: String,
    #[serde(rename = "Name")]
    pub name: Option<String>,
    #[serde(rename = "Email")]
    pub email: String,
}

impl PassClient {
    pub async fn get_info(&self) -> Result<UserInfo> {
        let res = self.send(GET!("/core/v4/users")).await?;
        let response: GetUserResponse = assert_response!(res);
        Ok(UserInfo {
            user: UserInfoUser::from(response.user),
            env: self.client.env().clone(),
        })
    }

    pub async fn get_service_account_name(&self) -> Result<String> {
        let service_account_data = self.get_service_account_self().await?;

        let encrypted_name = crate::utils::b64_decode(&service_account_data.name)
            .context("Error decoding service account name")?;

        let service_account_key = self
            .get_local_service_account_key()
            .await
            .context("Error getting local service account key")?;

        let decrypted_name = crypto::decrypt(
            &encrypted_name,
            &service_account_key,
            crypto::EncryptionTag::ServiceAccountName,
        )
        .map_err(|e| anyhow::anyhow!("Error decrypting service account name: {:?}", e))?;

        String::from_utf8(decrypted_name).context("Service account name is not valid UTF-8")
    }

    async fn get_service_account_self(&self) -> Result<ServiceAccountSelfData> {
        let res = self.send(GET!("/pass/v1/service_account/self")).await?;
        let response: ServiceAccountSelfResponse = assert_response!(res);
        Ok(response.service_account)
    }
}

#[derive(Debug, serde::Deserialize)]
struct ServiceAccountSelfResponse {
    #[serde(rename = "ServiceAccount")]
    service_account: ServiceAccountSelfData,
}

#[derive(Debug, serde::Deserialize)]
struct ServiceAccountSelfData {
    #[serde(rename = "ServiceAccountID")]
    #[allow(dead_code)]
    pub service_account_id: String,
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "ExpireTime")]
    #[allow(dead_code)]
    pub expire_time: Option<i64>,
}
