use crate::ShareId;
use crate::protos::folder::folder_v1;
use anyhow::{Context, Result};

#[derive(Clone, Debug, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
pub struct FolderId(pub(crate) String);
display_for_basic!(FolderId);

impl FolderId {
    pub fn new(id: String) -> Self {
        Self(id)
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Folder {
    pub id: FolderId,
    pub share_id: ShareId,
    pub parent_folder_id: Option<FolderId>,
    pub content: FolderData,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
pub struct FolderData {
    pub name: String,
}

impl FolderData {
    pub fn new(name: String) -> Self {
        Self { name }
    }

    pub fn serialize(self) -> Result<Vec<u8>> {
        let as_proto = folder_v1::Folder::from(self);
        as_proto
            .to_vec()
            .context("Error serializing folder to proto")
    }

    pub fn deserialize(data: &[u8]) -> Result<Self> {
        let as_proto = folder_v1::Folder::decode_from_slice(data)
            .context("Error decoding Folder from proto")?;
        Ok(Self::from(as_proto))
    }
}

impl From<FolderData> for folder_v1::Folder {
    fn from(value: FolderData) -> Self {
        folder_v1::Folder {
            name: value.name,
            ..Default::default()
        }
    }
}

impl From<folder_v1::Folder> for FolderData {
    fn from(value: folder_v1::Folder) -> Self {
        Self { name: value.name }
    }
}
