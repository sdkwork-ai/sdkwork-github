use sdkwork_github_integration_service::GitHubIntegrationService;
use sdkwork_github_integration_repository_sqlx::SqlGitHubStore;

pub async fn maybe_bootstrap_notable_catalog(
    service: &GitHubIntegrationService<SqlGitHubStore>,
) -> Result<(), String> {
    let enabled = std::env::var("SDKWORK_GITHUB_CATALOG_SYNC_ON_BOOT")
        .map(|value| matches!(value.trim(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false);
    if !enabled {
        return Ok(());
    }

    let tenant_id = std::env::var("SDKWORK_GITHUB_CATALOG_TENANT_ID")
        .unwrap_or_else(|_| "100001".to_string());
    let organization_id = std::env::var("SDKWORK_GITHUB_CATALOG_ORGANIZATION_ID")
        .unwrap_or_else(|_| "0".to_string());

    let result = match service
        .bootstrap_notable_catalog(&tenant_id, &organization_id)
        .await
    {
        Ok(result) => result,
        Err(error) => {
            tracing::warn!(
                tenant_id,
                organization_id,
                error = %error,
                "notable GitHub repository catalog bootstrap skipped during startup"
            );
            return Ok(());
        }
    };

    tracing::info!(
        tenant_id,
        organization_id,
        repositories_synced = result.repositories_synced,
        issues_synced = result.issues_synced,
        plans_created = result.plans_created,
        plan_items_created = result.plan_items_created,
        "notable GitHub repository catalog bootstrap completed"
    );
    Ok(())
}
