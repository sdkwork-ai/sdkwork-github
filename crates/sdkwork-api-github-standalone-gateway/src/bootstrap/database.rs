use sdkwork_github_database_host::bootstrap_github_database_from_env;
use sdkwork_github_integration_repository_sqlx::SqlGitHubStore;
use sdkwork_github_integration_service::GitHubIntegrationService;
use sdkwork_database_sqlx::DatabasePool;

pub struct GitHubBootstrap {
    pub service: GitHubIntegrationService<SqlGitHubStore>,
    pub pool: DatabasePool,
}

pub async fn build_github_bootstrap() -> Result<GitHubBootstrap, String> {
    let host = bootstrap_github_database_from_env().await?;
    let pool = host.pool().clone();
    let service = GitHubIntegrationService::new(SqlGitHubStore::new(pool.clone()));
    super::catalog::maybe_bootstrap_notable_catalog(&service).await?;
    Ok(GitHubBootstrap { service, pool })
}

pub async fn build_github_service(
) -> Result<GitHubIntegrationService<SqlGitHubStore>, String> {
    Ok(build_github_bootstrap().await?.service)
}
