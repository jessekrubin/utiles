//! Request ID middleware for using radix36 encoded request IDs (like Fastify).

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use axum::http::Request;
use tower_http::request_id::{MakeRequestId, RequestId};

use super::radix36::u64_radix36;

#[derive(Clone, Default)]
pub struct Radix36MakeRequestId {
    counter: Arc<AtomicU64>,
}

impl MakeRequestId for Radix36MakeRequestId {
    fn make_request_id<B>(&mut self, _request: &Request<B>) -> Option<RequestId> {
        let request_id_u64 = self.counter.fetch_add(1, Ordering::SeqCst);
        let request_id = u64_radix36(request_id_u64)
            .parse()
            .expect("Failed to parse request_id");
        Some(RequestId::new(request_id))
    }
}
