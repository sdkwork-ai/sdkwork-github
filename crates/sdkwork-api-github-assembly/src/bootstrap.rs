//! Gateway bootstrap for sdkwork-github.

use std::sync::Arc;

use axum::Router;
use sdkwork_github_integration_repository_sqlx::SqlGitHubStore;
use sdkwork_github_integration_service::{
    ports::{GitHubSyncStore, TrackerStore}, GitHubIntegrationService,
};
pub use sdkwork_web_bootstrap::ApiAssemblyContribution;
use sdkwork_web_bootstrap::{AlwaysReady, HttpRouteManifest};

use crate::catalog::maybe_bootstrap_notable_catalog;
use crate::readiness::GithubDatabaseReadinessCheck;

/// Indivisible host-neutral API assembly contribution (web-bootstrap contract).
pub type ApiAssembly = ApiAssemblyContribution;

/// Boots the GitHub database, integration service, and optional notable
/// catalog from the process environment, then assembles the complete
/// host-neutral contribution (API_ASSEMBLY_SPEC §3/§6.1). Consuming gateways
/// call this entrypoint instead of importing `sdkwork-github-database-host`,
/// `sdkwork-github-integration-service`, or
/// `sdkwork-github-integration-repository-sqlx` directly.
pub async fn assemble_api_router_from_env() -> Result<ApiAssembly, String> {
    let host = sdkwork_github_database_host::bootstrap_github_database_from_env().await?;
    let pool = host.pool().clone();
    let service = GitHubIntegrationService::new(SqlGitHubStore::new(pool.clone()));
    maybe_bootstrap_notable_catalog(&service).await?;
    let mut assembly = assemble_api_router(service);
    assembly.readiness_check = Arc::new(GithubDatabaseReadinessCheck::new(pool));
    Ok(assembly)
}

pub fn assemble_api_router<S>(service: GitHubIntegrationService<S>) -> ApiAssembly
where
    S: GitHubSyncStore + TrackerStore + Clone + Send + Sync + 'static,
{
    let app_router = sdkwork_routes_github_app_api::gateway_mount(service.clone());
    let backend_router = sdkwork_routes_github_backend_api::gateway_mount(service);
    let router = Router::new().merge(app_router).merge(backend_router);
    let routes = [
        sdkwork_routes_github_app_api::gateway_route_manifest(),
        sdkwork_routes_github_backend_api::gateway_route_manifest(),
    ]
    .into_iter()
    .flat_map(|manifest| manifest.routes().to_vec())
    .collect();
    ApiAssemblyContribution::from_manifest(
        "sdkwork-github",
        "SDKWork GitHub API",
        router,
        HttpRouteManifest::from_owned_routes(routes),
        Vec::new(),
        Arc::new(AlwaysReady),
    )
    .expect("sdkwork-github API assembly contribution must be valid")
}
