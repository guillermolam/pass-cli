use crate::PassClient;
use crate::item::list::ItemRevision;
use anyhow::{Context, Result};
use muon::POST;
use pass_domain::{ItemContent, ItemId, LoginItem, ShareId};

#[derive(Debug)]
pub struct LoginItemCreatePayload {
    pub title: String,
    pub email: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub urls: Vec<String>,
}

#[derive(serde::Deserialize)]
struct CreateItemResponse {
    #[serde(rename = "Item")]
    pub item: ItemRevision,
}

impl PassClient {
    pub async fn create_login(
        &self,
        share_id: &ShareId,
        payload: LoginItemCreatePayload,
    ) -> Result<ItemId> {
        let req = self
            .create_item_request(
                share_id,
                &payload.title,
                ItemContent::Login(LoginItem {
                    email: payload.email.unwrap_or_default(),
                    username: payload.username.unwrap_or_default(),
                    password: payload.password.unwrap_or_default(),
                    urls: payload.urls,
                    totp_uri: String::new(),
                }),
            )
            .await
            .context("Error creating login item request")?;

        let res = POST!("/pass/v1/share/{share_id}/item")
            .body_json(req)
            .context("Error serializing create_login request")?;
        let response = self
            .client
            .send(res)
            .await
            .context("Error sending create login request")?;
        let response: CreateItemResponse = assert_response!(response);

        Ok(ItemId::new(response.item.item_id))
    }
}
