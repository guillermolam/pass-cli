use crate::PassClient;
use crate::common::CodeResponse;
use anyhow::{Context, Result};
use muon::POST;
use pass_domain::{TelemetryEvent, TelemetryEventData};
use std::collections::HashMap;

const EVENT_CHUNK_SIZE: usize = 500;
const MEASUREMENT_GROUP: &str = "pass.any.user_actions";
const PLAN_NAME_KEY: &str = "user_tier";

#[derive(Debug, serde::Serialize)]
struct SendTelemetryRequest {
    #[serde(rename = "EventInfo")]
    event_info: Vec<EventInfo>,
}

#[derive(Debug, serde::Serialize)]
struct EventInfo {
    #[serde(rename = "MeasurementGroup")]
    measurement_group: String,
    #[serde(rename = "Event")]
    event: String,
    #[serde(rename = "Values")]
    values: HashMap<String, String>,
    #[serde(rename = "Dimensions")]
    dimensions: HashMap<String, String>,
}

impl PassClient {
    // Convenience method to emit telemetry events.
    // Failures are logged but not propagated to avoid breaking operations.
    pub async fn emit_telemetry(&self, event: &dyn TelemetryEvent) {
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

    pub async fn send_telemetry_events(&self, events: Vec<TelemetryEventData>) -> Result<()> {
        let plan = self
            .get_user_access()
            .await
            .context("Error getting user access")?
            .plan
            .internal_name;

        let mut extra_dimensions = Self::get_os_info();
        extra_dimensions.insert(PLAN_NAME_KEY.to_string(), plan);

        let chunks = events.chunks(EVENT_CHUNK_SIZE);
        for chunk in chunks {
            self.send_telemetry_chunk(&extra_dimensions, chunk).await?;
        }
        Ok(())
    }

    async fn send_telemetry_chunk(
        &self,
        extra_dimensions: &HashMap<String, String>,
        chunk: &[TelemetryEventData],
    ) -> Result<()> {
        let body = Self::build_request(extra_dimensions, chunk);
        let req = POST!("/data/v1/stats/multiple")
            .body_json(body)
            .context("Error creating telemetry request")?;

        let res = self.send(req).await?;
        let response: CodeResponse = assert_response!(res);
        response.success_guard()?;

        Ok(())
    }

    fn build_request(
        extra_dimensions: &HashMap<String, String>,
        chunk: &[TelemetryEventData],
    ) -> SendTelemetryRequest {
        let mut events = Vec::new();

        for event in chunk {
            let event_info = Self::build_event(extra_dimensions, event);
            events.push(event_info);
        }

        SendTelemetryRequest { event_info: events }
    }

    fn build_event(
        extra_dimensions: &HashMap<String, String>,
        event: &TelemetryEventData,
    ) -> EventInfo {
        let mut dimensions = event.dimensions.clone();
        for (name, value) in extra_dimensions {
            dimensions.insert(name.to_string(), value.to_string());
        }

        EventInfo {
            measurement_group: MEASUREMENT_GROUP.to_string(),
            event: event.event_type.clone(),
            values: HashMap::new(), // unused
            dimensions,
        }
    }

    fn get_os_info() -> HashMap<String, String> {
        let mut os_info: HashMap<String, String> = HashMap::new();
        os_info.insert("os".to_string(), std::env::consts::OS.to_string());
        os_info.insert("arch".to_string(), std::env::consts::ARCH.to_string());
        os_info
    }
}
