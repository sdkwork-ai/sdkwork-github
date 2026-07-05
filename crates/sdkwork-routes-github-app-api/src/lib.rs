pub mod dto;
pub mod handlers;
pub mod http_route_manifest;
pub mod paths;
pub mod routes;
pub mod state;
pub mod web_bootstrap;

pub use http_route_manifest::{app_route_manifest, APP_HTTP_ROUTES};
pub use web_bootstrap::{
    github_app_public_path_prefixes, wrap_router_with_dev_web_framework,
    wrap_router_with_web_framework, wrap_router_with_web_framework_from_env,
};

pub fn gateway_route_manifest() -> sdkwork_web_core::HttpRouteManifest {
    app_route_manifest()
}

pub fn gateway_mount<S>(service: sdkwork_github_integration_service::GitHubIntegrationService<S>) -> axum::Router
where
    S: sdkwork_github_integration_service::ports::GitHubSyncStore
        + sdkwork_github_integration_service::ports::TrackerStore
        + Clone
        + Send
        + Sync
        + 'static,
{
    routes::build_router(service)
}
