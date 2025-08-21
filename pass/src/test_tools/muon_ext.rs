use crate::PassClient;
use crate::test_tools::client_features::TestClientFeatures;
use muon::test::server::{Request, Response, Server};
use std::sync::Arc;

pub trait MuonServerExt {
    fn handler<P, F>(&self, path: P, handler: F)
    where
        P: AsRef<str> + Send + Sync + 'static,
        F: Fn(&Request) -> Option<Response> + Send + Sync + 'static;

    fn pass_client(&self) -> PassClient;
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

    fn pass_client(&self) -> PassClient {
        let key = pass_domain::crypto::generate_encryption_key();
        PassClient::new(self.client(), Arc::new(TestClientFeatures::new(key)))
    }
}

pub fn new_res<B: Default>(status: u16) -> Option<Response<B>> {
    Some(
        Response::builder()
            .status(status)
            .body(B::default())
            .unwrap(),
    )
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
