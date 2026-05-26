/*
 *  Copyright (c) 2026 Proton AG
 *  This file is part of Proton AG and Proton Pass.
 *
 *  Proton Pass is free software: you can redistribute it and/or modify
 *  it under the terms of the GNU General Public License as published by
 *  the Free Software Foundation, either version 3 of the License, or
 *  (at your option) any later version.
 *
 *  Proton Pass is distributed in the hope that it will be useful,
 *  but WITHOUT ANY WARRANTY; without even the implied warranty of
 *  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 *  GNU General Public License for more details.
 *
 *  You should have received a copy of the GNU General Public License
 *  along with Proton Pass.  If not, see <https://www.gnu.org/licenses/>.
 *
 */

use crate::user_keys::USER_KEYS_FILE_NAME;
use crate::{PassClient, PassClientContext};
use anyhow::{Context, Result};
use muon::GET;
use pass_domain::AccountType;
use std::path::Path;

const CURSOR_FILE_NAME: &str = "core_event_cursor";

/// Cache key for a cursor update that must be persisted after a successful key refresh.
#[derive(Clone)]
pub(crate) struct PendingCursorUpdateCacheType;

#[derive(serde::Deserialize)]
struct LatestEventResponse {
    #[serde(rename = "EventID")]
    event_id: String,
}

#[derive(serde::Deserialize)]
struct CoreEventsResponse {
    #[serde(rename = "EventID")]
    event_id: String,
    #[serde(rename = "More")]
    more: u8,
    #[serde(rename = "User")]
    user: Option<CoreUserEvent>,
}

#[derive(serde::Deserialize)]
struct CoreUserEvent {
    #[serde(rename = "Keys", default)]
    keys: Vec<serde_json::Value>,
}

pub(crate) async fn read_cursor<C: PassClientContext>(
    client: &PassClient<C>,
) -> Result<Option<String>> {
    let fs = client.client_features.get_fs().await;
    let path = Path::new(CURSOR_FILE_NAME);
    if !fs
        .file_exists(path)
        .await
        .context("Error checking event cursor file")?
    {
        return Ok(None);
    }
    let bytes = fs
        .get_file(path)
        .await
        .context("Error reading event cursor file")?;
    Ok(Some(
        String::from_utf8(bytes).context("Invalid event cursor encoding")?,
    ))
}

pub(crate) async fn write_cursor<C: PassClientContext>(
    client: &PassClient<C>,
    event_id: &str,
) -> Result<()> {
    let fs = client.client_features.get_fs().await;
    fs.store_file(event_id.as_bytes().to_vec(), Path::new(CURSOR_FILE_NAME))
        .await
        .context("Error writing event cursor file")
}

/// Called once at CLI bootstrap (after session load, before commands dispatch).
/// Checks for key changes since last run and, if found, invalidates the key cache
/// so the next `get_user_keys()` call re-fetches from the API.
/// No-ops for PAT and agent sessions.
pub async fn bootstrap_event_sync<C: PassClientContext>(client: &PassClient<C>) {
    if client.account_type() == AccountType::PersonalAccessToken
        || client.account_type() == AccountType::AgentSession
    {
        return;
    }

    match sync_core_events(client).await {
        Ok(SyncResult {
            keys_changed: true,
            new_event_id,
        }) => {
            debug!("User keys changed during bootstrap, invalidating key cache");
            let fs = client.client_features.get_fs().await;
            fs.remove_file(Path::new(USER_KEYS_FILE_NAME)).await.ok();
            // Store the new event_id as a pending update; get_user_keys() will commit it
            // after confirming keys were successfully refreshed (golden rule).
            client
                .cache
                .store(PendingCursorUpdateCacheType, new_event_id)
                .await;
        }
        Ok(_) => {} // cursor already written inside sync_core_events for no-change path
        Err(e) => warn!("Failed to sync core events during bootstrap: {e:#}"),
    }
}

struct SyncResult {
    keys_changed: bool,
    new_event_id: String,
}

async fn sync_core_events<C: PassClientContext>(client: &PassClient<C>) -> Result<SyncResult> {
    let mut current_id = match read_cursor(client).await? {
        None => {
            debug!("No core event cursor stored, fetching latest event ID");
            let res = client.send(GET!("/core/v4/events/latest")).await?;
            let response: LatestEventResponse = assert_response!(res);
            debug!(
                "Seeding core event cursor with event_id={}",
                response.event_id
            );
            write_cursor(client, &response.event_id).await?;
            return Ok(SyncResult {
                keys_changed: false,
                new_event_id: response.event_id,
            });
        }
        Some(id) => id,
    };

    let mut keys_changed = false;
    debug!("Fetching core events since event_id={current_id}");
    loop {
        let res = client.send(GET!("/core/v4/events/{current_id}")).await?;
        let response: CoreEventsResponse = assert_response!(res);

        if let Some(ref user) = response.user
            && !user.keys.is_empty()
        {
            debug!("Core event contains key changes, will refresh user keys");
            keys_changed = true;
        }

        current_id = response.event_id;

        if response.more == 0 {
            break;
        }
        debug!("More core events pending, fetching next page from event_id={current_id}");
    }

    if !keys_changed {
        write_cursor(client, &current_id).await?;
    }

    Ok(SyncResult {
        keys_changed,
        new_event_id: current_id,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_tools::*;

    #[muon_test::test]
    async fn first_run_seeds_cursor_without_key_refresh(server: muon_test::Server) {
        let (raw_client, api) = server.client::<()>();
        let client = make_test_pass_client(raw_client, &api).await;

        let handled = api.handler_with_method(Method::GET, "/core/v4/events/latest", move |_| {
            success(serde_json::json!({ "Code": 1000, "EventID": "event-initial" }))
        });

        bootstrap_event_sync(&client).await;

        assert_hit!(handled);
        let pending = client
            .cache
            .get::<PendingCursorUpdateCacheType, String>(PendingCursorUpdateCacheType)
            .await;
        assert!(pending.is_none());
        assert_eq!(
            Some("event-initial".to_string()),
            read_cursor(&client).await.unwrap()
        );
    }

    #[muon_test::test]
    async fn subsequent_run_no_key_changes(server: muon_test::Server) {
        let (raw_client, api) = server.client::<()>();
        let client = make_test_pass_client(raw_client, &api).await;

        write_cursor(&client, "event-abc").await.unwrap();

        let handled =
            api.handler_with_method(Method::GET, "/core/v4/events/event-abc", move |_| {
                success(serde_json::json!({ "Code": 1000, "EventID": "event-xyz", "More": 0 }))
            });

        bootstrap_event_sync(&client).await;

        assert_hit!(handled);
        let pending = client
            .cache
            .get::<PendingCursorUpdateCacheType, String>(PendingCursorUpdateCacheType)
            .await;
        assert!(pending.is_none());
        assert_eq!(
            Some("event-xyz".to_string()),
            read_cursor(&client).await.unwrap()
        );
    }

    #[muon_test::test]
    async fn subsequent_run_keys_changed(server: muon_test::Server) {
        let (raw_client, api) = server.client::<()>();
        let client = make_test_pass_client(raw_client, &api).await;

        write_cursor(&client, "event-abc").await.unwrap();

        let handled =
            api.handler_with_method(Method::GET, "/core/v4/events/event-abc", move |_| {
                success(serde_json::json!({
                    "Code": 1000, "EventID": "event-xyz", "More": 0,
                    "User": { "Keys": [{}] }
                }))
            });

        bootstrap_event_sync(&client).await;

        assert_hit!(handled);
        let pending = client
            .cache
            .get::<PendingCursorUpdateCacheType, String>(PendingCursorUpdateCacheType)
            .await;
        assert_eq!(Some("event-xyz".to_string()), pending);
        // Cursor file unchanged — waiting for key refresh
        assert_eq!(
            Some("event-abc".to_string()),
            read_cursor(&client).await.unwrap()
        );
    }
}
