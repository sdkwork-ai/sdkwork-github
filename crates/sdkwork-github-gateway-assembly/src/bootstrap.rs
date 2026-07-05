//! Gateway bootstrap for sdkwork-github.

use axum::Router;
use sdkwork_github_integration_service::{
    ports::{GitHubSyncStore, TrackerStore}, GitHubIntegrationService,
};

pub struct ApplicationAssembly {
    pub router: Router,
}

pub fn assemble_application_business_router<S>(service: GitHubIntegrationService<S>) -> ApplicationAssembly
where
    S: GitHubSyncStore + TrackerStore + Clone + Send + Sync + 'static,
{
    let app_router = sdkwork_routes_github_app_api::gateway_mount(service.clone());
    let backend_router = sdkwork_routes_github_backend_api::gateway_mount(service);
    ApplicationAssembly {
        router: Router::new().merge(app_router).merge(backend_router),
    }
}
