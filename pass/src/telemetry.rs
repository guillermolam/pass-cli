use std::collections::HashMap;
use crate::PassClient;
use anyhow::{Context, Result};
use muon::POST;
use pass_domain::TelemetryEvent;
use crate::common::CodeResponse;

const EVENT_CHUNK_SIZE: usize = 500;
const MEASUREMENT_GROUP: &str = "pass.any.user_actions";
const PLAN_NAME_KEY: &str = "user_tier";


#[derive(Debug, serde::Serialize)]
struct SendTelemetryRequest {
    #[serde(rename = "EventInfo")]
    event_info: Vec<EventInfo>
}

#[derive(Debug, serde::Serialize)]
struct EventInfo {
    #[serde(rename = "MeasurementGroup")]
    measurement_group: String,
    #[serde(rename = "Event")]
    event: String,
    #[serde(rename = "Values")]
    values: HashMap<String, serde_json::Value>,
    #[serde(rename = "Dimensions")]
    dimensions: HashMap<String, serde_json::Value>,
}

impl PassClient {
    // Convenience method to emit telemetry events.
    // Failures are logged but not propagated to avoid breaking operations.
    pub async fn emit_telemetry(&self, event: TelemetryEvent) {
        match self
            .client_features
            .get_telemetry_handler()
            .await
            .emit_telemetry(event)
            .await
        {
            Ok(_) => {}
            Err(e) => {
                warn!("Failed to emit telemetry event: {:?}", e);
            }
        }
    }

    pub async fn send_telemetry_events(&self, events: Vec<TelemetryEvent>) -> Result<()> {
        let plan = self.get_user_access()
            .await
            .context("Error getting user access")?
            .plan
            .internal_name;
        let chunks = events.chunks(EVENT_CHUNK_SIZE);
        for chunk in chunks {
            self.send_telemetry_chunk(&plan, chunk).await?;
        }
        Ok(())
    }

    async fn send_telemetry_chunk(&self, plan: &str, chunk: &[TelemetryEvent]) -> Result<()> {
        let body = Self::build_request(plan, chunk);
        let req = POST!("/data/v1/stats/multiple")
            .body_json(body)
            .context("Error creating telemetry request")?;

        let res = self.send(req).await?;
        let response: CodeResponse = assert_response!(res);
        response.success_guard()?;

        Ok(())
    }

    fn build_request(plan: &str, chunk: &[TelemetryEvent]) -> SendTelemetryRequest {
        let mut events = Vec::new();

        for event in chunk {
            let event_info = Self::build_event(plan, event);
            events.push(event_info);
        }

        SendTelemetryRequest { event_info: events }
    }

    fn build_event(plan: &str, event: &TelemetryEvent) -> EventInfo {
        let mut dimensions = HashMap::new();
        dimensions.insert(PLAN_NAME_KEY.to_string(), serde_json::Value::String(plan.to_string()));
        EventInfo {
            measurement_group: MEASUREMENT_GROUP.to_string(),
            event: event.event_type().to_string(),
            values: HashMap::new(),
            dimensions,
        }
    }
}
