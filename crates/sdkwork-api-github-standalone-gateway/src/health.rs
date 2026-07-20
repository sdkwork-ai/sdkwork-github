//! Readiness and metrics assembly for the GitHub standalone gateway.
//!
//! Per `WEB_BACKEND_SPEC.md`, this module assembles `ReadinessCheck` implementations
//! and observability snapshots. Infra routes (`/healthz`, `/readyz`, `/livez`, `/metrics`)
//! are mounted through `sdkwork-web-bootstrap::service_router`.

use std::sync::{Arc, OnceLock};

use sdkwork_database_sqlx::DatabasePool;
use sdkwork_github_integration_provider_github::{
    provider_metrics_snapshot, ProviderMetricsSnapshot,
};
use sdkwork_web_bootstrap::ReadinessCheck;
use sdkwork_web_core::HttpMetricsRegistry;

use crate::readiness::GithubDatabaseReadinessCheck;

static HTTP_METRICS: OnceLock<Arc<HttpMetricsRegistry>> = OnceLock::new();

pub fn http_metrics_registry() -> Arc<HttpMetricsRegistry> {
    HTTP_METRICS
        .get_or_init(HttpMetricsRegistry::new)
        .clone()
}

pub fn ready_check(pool: DatabasePool) -> Arc<dyn ReadinessCheck> {
    Arc::new(GithubDatabaseReadinessCheck::new(pool))
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct GithubMetricsSnapshot {
    pub http_prometheus: String,
    pub provider: ProviderMetricsSnapshot,
}

pub fn metrics_snapshot() -> GithubMetricsSnapshot {
    GithubMetricsSnapshot {
        http_prometheus: http_metrics_registry().render_prometheus(),
        provider: provider_metrics_snapshot(),
    }
}
