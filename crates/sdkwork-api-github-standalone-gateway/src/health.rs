//! Readiness and metrics assembly for the GitHub standalone gateway.
//!
//! Per `WEB_BACKEND_SPEC.md`, this module assembles `ReadinessCheck` implementations
//! and observability snapshots. Infra routes (`/healthz`, `/readyz`, `/livez`, `/metrics`)
//! are mounted through `sdkwork-web-bootstrap::service_router`.

use std::sync::{Arc, OnceLock};

use sdkwork_web_core::HttpMetricsRegistry;

static HTTP_METRICS: OnceLock<Arc<HttpMetricsRegistry>> = OnceLock::new();

pub fn http_metrics_registry() -> Arc<HttpMetricsRegistry> {
    HTTP_METRICS
        .get_or_init(HttpMetricsRegistry::new)
        .clone()
}
