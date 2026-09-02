use std::sync::Arc;

use axum::Router;

use crate::bootstrap::auth::build_protected_router;
use crate::health::http_metrics_registry;
use sdkwork_web_bootstrap::{ApiModuleRegistry, CompositeReadinessCheck, service_router, ServiceRouterConfig};

pub async fn build_router() -> Result<Router, Box<dyn std::error::Error + Send + Sync>> {
    // GitHub database, service, catalog, and route mounting are owned by the
    // GitHub API assembly; the IAM App API surface enters through the IAM API
    // assembly contribution (API_ASSEMBLY_SPEC §3/§6.1).
    let mut module_registry = ApiModuleRegistry::new();
    module_registry.add_module(sdkwork_api_github_assembly::assemble_api_router_from_env()
        .await
        .map_err(|error| -> Box<dyn std::error::Error + Send + Sync> { error.into() })?);
    let github = module_registry.try_compose("SDKWork Github API")?;
    let iam = sdkwork_api_iam_assembly::assemble_app_api_contribution()
        .await
        .map_err(|error| -> Box<dyn std::error::Error + Send + Sync> { error.into() })?;

    let domain = build_protected_router(github.router).await;

    let business = Router::new()
        .merge(iam.router)
        .merge(build_protected_router(domain).await)
        .layer(sdkwork_web_bootstrap::application_cors_layer_from_env(
            &["SDKWORK_GITHUB_ENVIRONMENT", "GITHUB_ENVIRONMENT"],
            &["SDKWORK_CORS_ALLOWED_ORIGINS"],
        ));

    let readiness = Arc::new(CompositeReadinessCheck::new(vec![
        github.readiness_check.clone(),
        iam.readiness_check.clone(),
    ]));

    Ok(service_router(
        business,
        ServiceRouterConfig::default()
            .with_readiness_check(readiness)
            .with_metrics(http_metrics_registry()),
    ))
}
