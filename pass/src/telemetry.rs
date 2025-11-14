use crate::PassClient;
use anyhow::Result;
use pass_domain::TelemetryEvent;

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

    pub async fn send_telemetry_events(&self, _events: Vec<TelemetryEvent>) -> Result<()> {
        // TBD: Implement telemetry sending logic
        Ok(())
    }
}
