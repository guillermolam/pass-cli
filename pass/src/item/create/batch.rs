use super::common::CreateItemRequest;
use crate::item::list::ItemRevision;
use crate::{PassClient, PassClientContext};
use anyhow::{Context, Result};
use muon::POST;
use pass_domain::{ItemData, ItemId, ShareId};

const MAX_ITEMS_PER_BATCH: usize = 100;

#[derive(serde::Serialize)]
struct BatchImportItemBody {
    #[serde(rename = "Item")]
    item: CreateItemRequest,
    #[serde(rename = "AliasEmail")]
    alias_email: Option<String>,
    #[serde(rename = "Trashed")]
    trashed: bool,
    #[serde(rename = "CreateTime")]
    create_time: Option<u64>,
    #[serde(rename = "ModifyTime")]
    modify_time: Option<u64>,
}

#[derive(serde::Serialize)]
struct BatchCreateItemsRequest {
    #[serde(rename = "Items")]
    items: Vec<BatchImportItemBody>,
}

#[derive(serde::Deserialize)]
struct BatchCreateItemsResponse {
    #[serde(rename = "Revisions")]
    revisions: BatchItemRevisions,
}

#[derive(serde::Deserialize)]
struct BatchItemRevisions {
    #[serde(rename = "RevisionsData")]
    items: Vec<ItemRevision>,
}

impl<C: PassClientContext> PassClient<C> {
    pub async fn create_items(
        &self,
        share_id: &ShareId,
        items: Vec<ItemData>,
    ) -> Result<Vec<ItemId>> {
        let mut all_ids = vec![];
        for chunk in items.chunks(MAX_ITEMS_PER_BATCH) {
            let ids = self.create_items_batch(share_id, chunk).await?;
            all_ids.extend(ids);
        }
        Ok(all_ids)
    }

    async fn create_items_batch(
        &self,
        share_id: &ShareId,
        items: &[ItemData],
    ) -> Result<Vec<ItemId>> {
        let mut import_bodies = vec![];
        for item in items {
            let req = self
                .create_item_request_from_data(share_id, item.clone(), None)
                .await
                .context("Error creating item request")?;
            import_bodies.push(BatchImportItemBody {
                item: req,
                alias_email: None,
                trashed: false,
                create_time: None,
                modify_time: None,
            });
        }

        let request = BatchCreateItemsRequest {
            items: import_bodies,
        };
        let res = POST!("/pass/v1/share/{share_id}/item/import/batch")
            .body_json(request)
            .context("Error serializing batch create items request")?;
        let response = self
            .send(res)
            .await
            .context("Error sending batch create items request")?;
        let response: BatchCreateItemsResponse = assert_response!(response);

        Ok(response
            .revisions
            .items
            .into_iter()
            .map(|r| ItemId::new(r.item_id))
            .collect())
    }
}
