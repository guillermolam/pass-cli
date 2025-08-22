use crate::PassClient;
use crate::test_tools::TEST_PASSPHRASE;
use crate::test_tools::client_features::TestClientFeatures;
use muon::test::server::{Request, Response, Server};
use std::sync::Arc;

pub trait MuonServerExt {
    fn handler<P, F>(&self, path: P, handler: F)
    where
        P: AsRef<str> + Send + Sync + 'static,
        F: Fn(&Request) -> Option<Response> + Send + Sync + 'static;

    async fn pass_client(&self) -> PassClient;
    fn pass_client_no_setup(&self) -> PassClient;
}

impl MuonServerExt for Arc<Server> {
    fn handler<P, F>(&self, path: P, handler: F)
    where
        P: AsRef<str> + Send + Sync + 'static,
        F: Fn(&Request) -> Option<Response> + Send + Sync + 'static,
    {
        self.add_handler(move |req| {
            if req.uri().path() == path.as_ref() {
                handler(req)
            } else {
                None
            }
        });
    }

    async fn pass_client(&self) -> PassClient {
        super::setup_user_data::setup(self);
        let client = self.pass_client_no_setup();

        client
            .setup_key_passphrases(TEST_PASSPHRASE)
            .await
            .expect("Error setting up passphrases");
        client
    }

    fn pass_client_no_setup(&self) -> PassClient {
        let key = pass_domain::crypto::generate_encryption_key();
        PassClient::new(self.client(), Arc::new(TestClientFeatures::new(key)))
    }
}

pub fn success<R: serde::Serialize>(res: R) -> Option<Response> {
    let body = serde_json::to_vec(&res).unwrap();
    Some(
        Response::builder()
            .status(200)
            .body(axum_core::body::Body::from(body))
            .unwrap(),
    )
}

#[macro_export]
macro_rules! last_request {
    ($recorder:expr) => {{
        let requests = $recorder.read();
        let req = requests
            .into_iter()
            .last()
            .expect("Failed to get last request");

        let bytes = req.body().to_vec();
        serde_json::from_slice(&bytes).expect("Failed to parse request")
    }};
}
