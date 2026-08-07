use async_trait::async_trait;
use chrono::Utc;
use sdkwork_database_config::DatabaseEngine;
use uuid::Uuid;

use sdkwork_github_integration_service::domain::{
    AdminIntegrationView, IntegrationStatus, Issue, LinkIntegrationCommand, Page, Plan, PlanItem,
    ProviderAccount, Repository,
};
use sdkwork_github_integration_service::error::ServiceError;
use sdkwork_github_integration_service::ports::GitHubSyncStore;

use super::store::{format_timestamp, parse_ts, SqlGitHubStore};

#[async_trait]
impl GitHubSyncStore for SqlGitHubStore {
    async fn upsert_repository(&self, repository: &Repository) -> Result<(), ServiceError> {
        let created_at = format_timestamp(repository.created_at);
        let updated_at = format_timestamp(repository.updated_at);
        let is_private = if repository.is_private { 1 } else { 0 };
        match self.pool().engine() {
            DatabaseEngine::Sqlite => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            DatabaseEngine::Postgres => {
                let pool = self.pool().as_postgres().expect("postgres pool");
                sqlx::query(
                    "INSERT INTO github_repository (id, tenant_id, organization_id, full_name, owner, description, default_branch, html_url, is_private, created_at, updated_at)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                     ON CONFLICT(id) DO UPDATE SET
                       full_name = EXCLUDED.full_name,
                       owner = EXCLUDED.owner,
                       description = EXCLUDED.description,
                       default_branch = EXCLUDED.default_branch,
                       html_url = EXCLUDED.html_url,
                       is_private = EXCLUDED.is_private,
                       updated_at = EXCLUDED.updated_at",
                )
                .bind(&repository.id)
                .bind(&repository.tenant_id)
                .bind(&repository.organization_id)
                .bind(&repository.full_name)
                .bind(&repository.owner)
                .bind(&repository.description)
                .bind(&repository.default_branch)
                .bind(&repository.html_url)
                .bind(is_private)
                .bind(created_at)
                .bind(updated_at)
                .execute(pool)
                .await
                .map_err(|error| ServiceError::Repository(error.to_string()))?;
            }
        }
        Ok(())
    }

    async fn upsert_issue(&self, issue: &Issue) -> Result<(), ServiceError> {
        let created_at = format_timestamp(issue.created_at);
        let updated_at = format_timestamp(issue.updated_at);
        match self.pool().engine() {
            DatabaseEngine::Sqlite => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            DatabaseEngine::Postgres => {
                let pool = self.pool().as_postgres().expect("postgres pool");
                sqlx::query(
                    "INSERT INTO github_issue (id, tenant_id, organization_id, repository_id, number, title, state, html_url, created_at, updated_at)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                     ON CONFLICT(id) DO UPDATE SET
                       title = EXCLUDED.title,
                       state = EXCLUDED.state,
                       html_url = EXCLUDED.html_url,
                       updated_at = EXCLUDED.updated_at",
                )
                .bind(&issue.id)
                .bind(&issue.tenant_id)
                .bind(&issue.organization_id)
                .bind(&issue.repository_id)
                .bind(issue.number)
                .bind(&issue.title)
                .bind(&issue.state)
                .bind(&issue.html_url)
                .bind(created_at)
                .bind(updated_at)
                .execute(pool)
                .await
                .map_err(|error| ServiceError::Repository(error.to_string()))?;
            }
        }
        Ok(())
    }

    async fn upsert_plan(&self, plan: &Plan) -> Result<(), ServiceError> {
        let created_at = format_timestamp(plan.created_at);
        let updated_at = format_timestamp(plan.updated_at);
        match self.pool().engine() {
            DatabaseEngine::Sqlite => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            DatabaseEngine::Postgres => {
                let pool = self.pool().as_postgres().expect("postgres pool");
                sqlx::query(
                    "INSERT INTO github_plan (id, tenant_id, organization_id, repository_id, title, status, created_at, updated_at)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                     ON CONFLICT(id) DO UPDATE SET
                       repository_id = EXCLUDED.repository_id,
                       title = EXCLUDED.title,
                       status = EXCLUDED.status,
                       updated_at = EXCLUDED.updated_at",
                )
                .bind(&plan.id)
                .bind(&plan.tenant_id)
                .bind(&plan.organization_id)
                .bind(&plan.repository_id)
                .bind(&plan.title)
                .bind(&plan.status)
                .bind(created_at)
                .bind(updated_at)
                .execute(pool)
                .await
                .map_err(|error| ServiceError::Repository(error.to_string()))?;
            }
        }
        Ok(())
    }

    async fn upsert_plan_item(&self, item: &PlanItem) -> Result<(), ServiceError> {
        let created_at = format_timestamp(item.created_at);
        let updated_at = format_timestamp(item.updated_at);
        match self.pool().engine() {
            DatabaseEngine::Sqlite => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            DatabaseEngine::Postgres => {
                let pool = self.pool().as_postgres().expect("postgres pool");
                sqlx::query(
                    "INSERT INTO github_plan_item (id, plan_id, title, status, sort_order, issue_id, created_at, updated_at)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                     ON CONFLICT(id) DO UPDATE SET
                       title = EXCLUDED.title,
                       status = EXCLUDED.status,
                       sort_order = EXCLUDED.sort_order,
                       issue_id = EXCLUDED.issue_id,
                       updated_at = EXCLUDED.updated_at",
                )
                .bind(&item.id)
                .bind(&item.plan_id)
                .bind(&item.title)
                .bind(&item.status)
                .bind(item.sort_order)
                .bind(&item.issue_id)
                .bind(created_at)
                .bind(updated_at)
                .execute(pool)
                .await
                .map_err(|error| ServiceError::Repository(error.to_string()))?;
            }
        }
        Ok(())
    }

    async fn find_active_provider_account(
        &self,
        tenant_id: &str,
        organization_id: &str,
        provider: &str,
    ) -> Result<Option<ProviderAccount>, ServiceError> {
        let row = match self.pool().engine() {
            DatabaseEngine::Sqlite => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            DatabaseEngine::Postgres => {
                let pool = self.pool().as_postgres().expect("postgres pool");
                sqlx::query_as::<_, ProviderAccountRow>(
                    "SELECT id, tenant_id, organization_id, provider, external_account_id, access_token_cipher, scopes, status, last_synced_at, created_at, updated_at
                     FROM github_provider_account
                     WHERE tenant_id = $1 AND organization_id = $2 AND provider = $3 AND status = 'active'
                     LIMIT 1",
                )
                .bind(tenant_id)
                .bind(organization_id)
                .bind(provider)
                .fetch_optional(pool)
                .await
                .map_err(|error| ServiceError::Repository(error.to_string()))?
            }
        };

        Ok(row.map(Into::into))
    }

    async fn upsert_provider_account(&self, account: &ProviderAccount) -> Result<(), ServiceError> {
        let created_at = format_timestamp(account.created_at);
        let updated_at = format_timestamp(account.updated_at);
        let last_synced_at = account.last_synced_at.map(format_timestamp);
        match self.pool().engine() {
            DatabaseEngine::Sqlite => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            DatabaseEngine::Postgres => {
                let pool = self.pool().as_postgres().expect("postgres pool");
                sqlx::query(
                    "INSERT INTO github_provider_account (id, tenant_id, organization_id, provider, external_account_id, access_token_cipher, scopes, status, last_synced_at, created_at, updated_at)
                     VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
                     ON CONFLICT(tenant_id, organization_id, provider) DO UPDATE SET
                       external_account_id = EXCLUDED.external_account_id,
                       access_token_cipher = EXCLUDED.access_token_cipher,
                       scopes = EXCLUDED.scopes,
                       status = EXCLUDED.status,
                       last_synced_at = EXCLUDED.last_synced_at,
                       updated_at = EXCLUDED.updated_at",
                )
                .bind(&account.id)
                .bind(&account.tenant_id)
                .bind(&account.organization_id)
                .bind(&account.provider)
                .bind(&account.external_account_id)
                .bind(&account.access_token_cipher)
                .bind(&account.scopes)
                .bind(&account.status)
                .bind(last_synced_at)
                .bind(created_at)
                .bind(updated_at)
                .execute(pool)
                .await
                .map_err(|error| ServiceError::Repository(error.to_string()))?;
            }
        }
        Ok(())
    }

    async fn revoke_provider_account(
        &self,
        tenant_id: &str,
        organization_id: &str,
        provider: &str,
    ) -> Result<(), ServiceError> {
        let updated_at = format_timestamp(Utc::now());
        match self.pool().engine() {
            DatabaseEngine::Sqlite => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            DatabaseEngine::Postgres => {
                let pool = self.pool().as_postgres().expect("postgres pool");
                sqlx::query(
                    "UPDATE github_provider_account
                     SET status = 'revoked', updated_at = $1
                     WHERE tenant_id = $2 AND organization_id = $3 AND provider = $4",
                )
                .bind(updated_at)
                .bind(tenant_id)
                .bind(organization_id)
                .bind(provider)
                .execute(pool)
                .await
                .map_err(|error| ServiceError::Repository(error.to_string()))?;
            }
        }
        Ok(())
    }

    async fn get_integration_status(
        &self,
        tenant_id: &str,
        organization_id: &str,
        provider: &str,
    ) -> Result<IntegrationStatus, ServiceError> {
        let account = self
            .find_active_provider_account(tenant_id, organization_id, provider)
            .await?;
        Ok(match account {
            Some(account) => IntegrationStatus {
                provider: account.provider,
                linked: true,
                status: Some(account.status),
                external_account_id: account.external_account_id,
                scopes: account.scopes,
                last_synced_at: account.last_synced_at,
            },
            None => IntegrationStatus {
                provider: provider.to_string(),
                linked: false,
                status: None,
                external_account_id: None,
                scopes: None,
                last_synced_at: None,
            },
        })
    }

    async fn link_integration(
        &self,
        tenant_id: &str,
        organization_id: &str,
        provider: &str,
        command: &LinkIntegrationCommand,
        access_token_cipher: &str,
    ) -> Result<IntegrationStatus, ServiceError> {
        let now = Utc::now();
        let account = ProviderAccount {
            id: format!("github-provider-{}", Uuid::new_v4()),
            tenant_id: tenant_id.to_string(),
            organization_id: organization_id.to_string(),
            provider: provider.to_string(),
            external_account_id: command.external_account_id.clone(),
            access_token_cipher: access_token_cipher.to_string(),
            scopes: command.scopes.clone(),
            status: "active".to_string(),
            last_synced_at: None,
            created_at: now,
            updated_at: now,
        };
        self.upsert_provider_account(&account).await?;
        self.get_integration_status(tenant_id, organization_id, provider)
            .await
    }

    async fn touch_provider_last_synced(
        &self,
        tenant_id: &str,
        organization_id: &str,
        provider: &str,
    ) -> Result<(), ServiceError> {
        let updated_at = format_timestamp(Utc::now());
        let last_synced_at = updated_at.clone();
        match self.pool().engine() {
            DatabaseEngine::Sqlite => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            DatabaseEngine::Postgres => {
                let pool = self.pool().as_postgres().expect("postgres pool");
                sqlx::query(
                    "UPDATE github_provider_account
                     SET last_synced_at = $1, updated_at = $2
                     WHERE tenant_id = $3 AND organization_id = $4 AND provider = $5 AND status = 'active'",
                )
                .bind(last_synced_at)
                .bind(updated_at)
                .bind(tenant_id)
                .bind(organization_id)
                .bind(provider)
                .execute(pool)
                .await
                .map_err(|error| ServiceError::Repository(error.to_string()))?;
            }
        }
        Ok(())
    }

    async fn create_oauth_pending(
        &self,
        state: &str,
        tenant_id: &str,
        organization_id: &str,
        expires_at: chrono::DateTime<Utc>,
    ) -> Result<(), ServiceError> {
        let created_at = format_timestamp(Utc::now());
        let expires_at = format_timestamp(expires_at);
        match self.pool().engine() {
            DatabaseEngine::Sqlite => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            DatabaseEngine::Postgres => {
                let pool = self.pool().as_postgres().expect("postgres pool");
                sqlx::query(
                    "INSERT INTO github_oauth_pending (state, tenant_id, organization_id, created_at, expires_at)
                     VALUES ($1, $2, $3, $4, $5)",
                )
                .bind(state)
                .bind(tenant_id)
                .bind(organization_id)
                .bind(created_at)
                .bind(expires_at)
                .execute(pool)
                .await
                .map_err(|error| ServiceError::Repository(error.to_string()))?;
            }
        }
        Ok(())
    }

    async fn consume_oauth_pending(
        &self,
        state: &str,
    ) -> Result<Option<(String, String)>, ServiceError> {
        let now = format_timestamp(Utc::now());
        let row = match self.pool().engine() {
            DatabaseEngine::Sqlite => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            DatabaseEngine::Postgres => {
                let pool = self.pool().as_postgres().expect("postgres pool");
                let row = sqlx::query_as::<_, OAuthPendingRow>(
                    "SELECT state, tenant_id, organization_id, created_at, expires_at
                     FROM github_oauth_pending
                     WHERE state = $1 AND expires_at >= $2
                     LIMIT 1",
                )
                .bind(state)
                .bind(now)
                .fetch_optional(pool)
                .await
                .map_err(|error| ServiceError::Repository(error.to_string()))?;
                if row.is_some() {
                    sqlx::query("DELETE FROM github_oauth_pending WHERE state = $1")
                        .bind(state)
                        .execute(pool)
                        .await
                        .map_err(|error| ServiceError::Repository(error.to_string()))?;
                }
                row
            }
        };
        Ok(row.map(|value| (value.tenant_id, value.organization_id)))
    }

    async fn purge_expired_oauth_pending(&self) -> Result<(), ServiceError> {
        let now = format_timestamp(Utc::now());
        match self.pool().engine() {
            DatabaseEngine::Sqlite => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            DatabaseEngine::Postgres => {
                let pool = self.pool().as_postgres().expect("postgres pool");
                sqlx::query("DELETE FROM github_oauth_pending WHERE expires_at < $1")
                    .bind(now)
                    .execute(pool)
                    .await
                    .map_err(|error| ServiceError::Repository(error.to_string()))?;
            }
        }
        Ok(())
    }

    async fn list_admin_integrations(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<Page<AdminIntegrationView>, ServiceError> {
        let offset = ((page.saturating_sub(1)) * page_size) as i64;
        let limit = page_size as i64;
        match self.pool().engine() {
            DatabaseEngine::Sqlite => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            DatabaseEngine::Postgres => {
                let pool = self.pool().as_postgres().expect("postgres pool");
                let total: (i64,) = sqlx::query_as(
                    "SELECT COUNT(*) FROM github_provider_account WHERE status = 'active'",
                )
                .fetch_one(pool)
                .await
                .map_err(|error| ServiceError::Repository(error.to_string()))?;
                let rows = sqlx::query_as::<_, AdminIntegrationRow>(
                    "SELECT tenant_id, organization_id, provider, external_account_id, scopes, status, last_synced_at
                     FROM github_provider_account
                     WHERE status = 'active'
                     ORDER BY updated_at DESC
                     LIMIT $1 OFFSET $2",
                )
                .bind(limit)
                .bind(offset)
                .fetch_all(pool)
                .await
                .map_err(|error| ServiceError::Repository(error.to_string()))?;
                Ok(Page {
                    items: rows.into_iter().map(Into::into).collect(),
                    page,
                    page_size,
                    total: total.0 as u64,
                })
            }
        }
    }
}

#[derive(sqlx::FromRow)]
struct OAuthPendingRow {
    #[allow(dead_code)]
    state: String,
    tenant_id: String,
    organization_id: String,
    #[allow(dead_code)]
    created_at: String,
    #[allow(dead_code)]
    expires_at: String,
}

#[derive(sqlx::FromRow)]
struct AdminIntegrationRow {
    tenant_id: String,
    organization_id: String,
    provider: String,
    external_account_id: Option<String>,
    scopes: Option<String>,
    status: String,
    last_synced_at: Option<String>,
}

impl From<AdminIntegrationRow> for AdminIntegrationView {
    fn from(row: AdminIntegrationRow) -> Self {
        Self {
            tenant_id: row.tenant_id,
            organization_id: row.organization_id,
            provider: row.provider,
            linked: row.status == "active",
            status: Some(row.status),
            external_account_id: row.external_account_id,
            scopes: row.scopes,
            last_synced_at: row.last_synced_at.map(|value| parse_ts(&value)),
        }
    }
}

#[derive(sqlx::FromRow)]
struct ProviderAccountRow {
    id: String,
    tenant_id: String,
    organization_id: String,
    provider: String,
    external_account_id: Option<String>,
    access_token_cipher: String,
    scopes: Option<String>,
    status: String,
    last_synced_at: Option<String>,
    created_at: String,
    updated_at: String,
}

impl From<ProviderAccountRow> for ProviderAccount {
    fn from(row: ProviderAccountRow) -> Self {
        Self {
            id: row.id,
            tenant_id: row.tenant_id,
            organization_id: row.organization_id,
            provider: row.provider,
            external_account_id: row.external_account_id,
            access_token_cipher: row.access_token_cipher,
            scopes: row.scopes,
            status: row.status,
            last_synced_at: row.last_synced_at.map(|value| parse_ts(&value)),
            created_at: parse_ts(&row.created_at),
            updated_at: parse_ts(&row.updated_at),
        }
    }
}
