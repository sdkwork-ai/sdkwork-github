//! Gateway bootstrap for sdkwork-github.

use axum::Router;
use sdkwork_github_integration_service::{
    ports::{GitHubSyncStore, TrackerStore}, GitHubIntegrationService,
};

pub struct ApiAssembly {
    pub router: Router,
}

pub fn assemble_business_router<S>(service: GitHubIntegrationService<S>) -> ApiAssembly
where
    S: GitHubSyncStore + TrackerStore + Clone + Send + Sync + 'static,
{
    let app_router = sdkwork_routes_github_app_api::gateway_mount(service.clone());
    let backend_router = sdkwork_routes_github_backend_api::gateway_mount(service);
    ApiAssembly {
        router: Router::new().merge(app_router).merge(backend_router),
    }
}
