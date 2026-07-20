use axum::Router;

use crate::bootstrap::auth::build_protected_router;
use crate::bootstrap::database::build_github_bootstrap;
use crate::health::{http_metrics_registry, ready_check};
use sdkwork_web_bootstrap::{service_router, ServiceRouterConfig};

pub async fn build_router() -> Result<Router, Box<dyn std::error::Error + Send + Sync>> {
    let bootstrap = build_github_bootstrap()
        .await
        .map_err(|error| -> Box<dyn std::error::Error + Send + Sync> { error.into() })?;
    let service = bootstrap.service.clone();
    let pool = bootstrap.pool.clone();

    sdkwork_iam_database_host::bootstrap_iam_database_from_env()
        .await
        .map_err(|error| -> Box<dyn std::error::Error + Send + Sync> { error.into() })?;
    crate::bootstrap::iam_application_bootstrap::ensure_github_tenant_application_bootstrap()
        .await
        .map_err(|error| -> Box<dyn std::error::Error + Send + Sync> { error.into() })?;

    let iam_router = sdkwork_routes_iam_app_api::build_sdkwork_iam_app_api_router()
        .await
        .map_err(|error| -> Box<dyn std::error::Error + Send + Sync> { error.into() })?;

    let domain =
        sdkwork_api_github_assembly::assemble_api_router(service.clone())
            .router;

    let protected = build_protected_router(domain).await;

    let business = Router::new()
        .merge(iam_router)
        .merge(build_protected_router(protected).await)
        .layer(sdkwork_web_bootstrap::application_cors_layer_from_env(
            &["SDKWORK_GITHUB_ENVIRONMENT", "GITHUB_ENVIRONMENT"],
            &[
                "SDKWORK_GITHUB_CORS_ALLOWED_ORIGINS",
                "SDKWORK_CORS_ALLOWED_ORIGINS",
            ],
        ));

    Ok(service_router(
        business,
        ServiceRouterConfig::default()
            .with_readiness_check(ready_check(pool))
            .with_metrics(http_metrics_registry()),
    ))
}
