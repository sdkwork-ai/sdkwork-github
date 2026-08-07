use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sdkwork_database_config::DatabaseEngine;
use sdkwork_database_sqlx::DatabasePool;

use sdkwork_github_integration_service::domain::{Issue, Page, Plan, PlanItem, Repository};
use sdkwork_github_integration_service::error::ServiceError;
use sdkwork_github_integration_service::ports::GitHubStore;

#[derive(Clone)]
pub struct SqlGitHubStore {
    pool: DatabasePool,
}

impl SqlGitHubStore {
    pub fn new(pool: DatabasePool) -> Self {
        Self { pool }
    }

    pub(crate) fn pool(&self) -> &DatabasePool {
        &self.pool
    }
}

#[async_trait]
impl GitHubStore for SqlGitHubStore {
    async fn list_repositories(
        &self,
        tenant_id: &str,
        organization_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<Page<Repository>, ServiceError> {
        let offset = ((page.saturating_sub(1)) * page_size) as i64;
        let limit = page_size as i64;
        match self.pool.engine() {
            DatabaseEngine::Sqlite => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            DatabaseEngine::Postgres => {
                let pool = self.pool.as_postgres().expect("postgres pool");
                let total: (i64,) = sqlx::query_as(
                    "SELECT COUNT(*) FROM github_repository WHERE tenant_id = $1 AND organization_id = $2",
                )
                .bind(tenant_id)
                .bind(organization_id)
                .fetch_one(pool)
                .await
                .map_err(|error| ServiceError::Repository(error.to_string()))?;

                let rows = sqlx::query_as::<_, RepositoryRow>(
                    "SELECT id, tenant_id, organization_id, full_name, owner, description, default_branch, html_url, is_private, created_at, updated_at
                     FROM github_repository WHERE tenant_id = $1 AND organization_id = $2
                     ORDER BY updated_at DESC LIMIT $3 OFFSET $4",
                )
                .bind(tenant_id)
                .bind(organization_id)
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

    async fn list_issues(
        &self,
        tenant_id: &str,
        organization_id: &str,
        repository_id: Option<&str>,
        page: u32,
        page_size: u32,
    ) -> Result<Page<Issue>, ServiceError> {
        let offset = ((page.saturating_sub(1)) * page_size) as i64;
        let limit = page_size as i64;
        match (self.pool.engine(), repository_id) {
            (DatabaseEngine::Sqlite, _) => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            (DatabaseEngine::Postgres, Some(repository_id)) => {
                let pool = self.pool.as_postgres().expect("postgres pool");
                let total: (i64,) = sqlx::query_as(
                    "SELECT COUNT(*) FROM github_issue WHERE tenant_id = $1 AND organization_id = $2 AND repository_id = $3",
                )
                .bind(tenant_id)
                .bind(organization_id)
                .bind(repository_id)
                .fetch_one(pool)
                .await
                .map_err(|error| ServiceError::Repository(error.to_string()))?;
                let rows = sqlx::query_as::<_, IssueRow>(
                    "SELECT id, tenant_id, organization_id, repository_id, number, title, state, html_url, created_at, updated_at
                     FROM github_issue WHERE tenant_id = $1 AND organization_id = $2 AND repository_id = $3
                     ORDER BY number DESC LIMIT $4 OFFSET $5",
                )
                .bind(tenant_id)
                .bind(organization_id)
                .bind(repository_id)
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
            (DatabaseEngine::Postgres, None) => {
                let pool = self.pool.as_postgres().expect("postgres pool");
                let total: (i64,) = sqlx::query_as(
                    "SELECT COUNT(*) FROM github_issue WHERE tenant_id = $1 AND organization_id = $2",
                )
                .bind(tenant_id)
                .bind(organization_id)
                .fetch_one(pool)
                .await
                .map_err(|error| ServiceError::Repository(error.to_string()))?;
                let rows = sqlx::query_as::<_, IssueRow>(
                    "SELECT id, tenant_id, organization_id, repository_id, number, title, state, html_url, created_at, updated_at
                     FROM github_issue WHERE tenant_id = $1 AND organization_id = $2
                     ORDER BY updated_at DESC LIMIT $3 OFFSET $4",
                )
                .bind(tenant_id)
                .bind(organization_id)
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

    async fn list_plans(
        &self,
        tenant_id: &str,
        organization_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<Page<Plan>, ServiceError> {
        let offset = ((page.saturating_sub(1)) * page_size) as i64;
        let limit = page_size as i64;
        match self.pool.engine() {
            DatabaseEngine::Sqlite => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            DatabaseEngine::Postgres => {
                let pool = self.pool.as_postgres().expect("postgres pool");
                let total: (i64,) = sqlx::query_as(
                    "SELECT COUNT(*) FROM github_plan WHERE tenant_id = $1 AND organization_id = $2",
                )
                .bind(tenant_id)
                .bind(organization_id)
                .fetch_one(pool)
                .await
                .map_err(|error| ServiceError::Repository(error.to_string()))?;

                let rows = sqlx::query_as::<_, PlanRow>(
                    "SELECT id, tenant_id, organization_id, repository_id, title, status, created_at, updated_at
                     FROM github_plan WHERE tenant_id = $1 AND organization_id = $2
                     ORDER BY updated_at DESC LIMIT $3 OFFSET $4",
                )
                .bind(tenant_id)
                .bind(organization_id)
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

    async fn list_plan_items_for_plan_ids(
        &self,
        plan_ids: &[String],
    ) -> Result<Vec<PlanItem>, ServiceError> {
        if plan_ids.is_empty() {
            return Ok(Vec::new());
        }
        match self.pool.engine() {
            DatabaseEngine::Sqlite => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            DatabaseEngine::Postgres => {
                let pool = self.pool.as_postgres().expect("postgres pool");
                let mut query = String::from(
                    "SELECT id, plan_id, title, status, sort_order, issue_id, created_at, updated_at
                     FROM github_plan_item WHERE plan_id = ANY(",
                );
                query.push_str("$1) ORDER BY plan_id ASC, sort_order ASC");
                sqlx::query_as::<_, PlanItemRow>(sqlx::AssertSqlSafe(query))
                    .bind(plan_ids)
                    .fetch_all(pool)
                    .await
                    .map_err(|error| ServiceError::Repository(error.to_string()))
                    .map(|rows| rows.into_iter().map(Into::into).collect())
            }
        }
    }
}

#[derive(sqlx::FromRow)]
struct RepositoryRow {
    id: String,
    tenant_id: String,
    organization_id: String,
    full_name: String,
    owner: String,
    description: Option<String>,
    default_branch: Option<String>,
    html_url: Option<String>,
    is_private: i64,
    created_at: String,
    updated_at: String,
}

impl From<RepositoryRow> for Repository {
    fn from(row: RepositoryRow) -> Self {
        Self {
            id: row.id,
            tenant_id: row.tenant_id,
            organization_id: row.organization_id,
            full_name: row.full_name,
            owner: row.owner,
            description: row.description,
            default_branch: row.default_branch,
            html_url: row.html_url,
            is_private: row.is_private != 0,
            created_at: parse_ts(&row.created_at),
            updated_at: parse_ts(&row.updated_at),
        }
    }
}

#[derive(sqlx::FromRow)]
struct IssueRow {
    id: String,
    tenant_id: String,
    organization_id: String,
    repository_id: String,
    number: i64,
    title: String,
    state: String,
    html_url: Option<String>,
    created_at: String,
    updated_at: String,
}

impl From<IssueRow> for Issue {
    fn from(row: IssueRow) -> Self {
        Self {
            id: row.id,
            tenant_id: row.tenant_id,
            organization_id: row.organization_id,
            repository_id: row.repository_id,
            number: row.number,
            title: row.title,
            state: row.state,
            html_url: row.html_url,
            created_at: parse_ts(&row.created_at),
            updated_at: parse_ts(&row.updated_at),
        }
    }
}

#[derive(sqlx::FromRow)]
struct PlanRow {
    id: String,
    tenant_id: String,
    organization_id: String,
    repository_id: Option<String>,
    title: String,
    status: String,
    created_at: String,
    updated_at: String,
}

impl From<PlanRow> for Plan {
    fn from(row: PlanRow) -> Self {
        Self {
            id: row.id,
            tenant_id: row.tenant_id,
            organization_id: row.organization_id,
            repository_id: row.repository_id,
            title: row.title,
            status: row.status,
            created_at: parse_ts(&row.created_at),
            updated_at: parse_ts(&row.updated_at),
        }
    }
}

#[derive(sqlx::FromRow)]
struct PlanItemRow {
    id: String,
    plan_id: String,
    title: String,
    status: String,
    sort_order: i32,
    issue_id: Option<String>,
    created_at: String,
    updated_at: String,
}

impl From<PlanItemRow> for PlanItem {
    fn from(row: PlanItemRow) -> Self {
        Self {
            id: row.id,
            plan_id: row.plan_id,
            title: row.title,
            status: row.status,
            sort_order: row.sort_order,
            issue_id: row.issue_id,
            created_at: parse_ts(&row.created_at),
            updated_at: parse_ts(&row.updated_at),
        }
    }
}

pub(crate) fn format_timestamp(value: DateTime<Utc>) -> String {
    value.to_rfc3339()
}

pub(crate) fn parse_ts(value: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(value)
        .map(|value| value.with_timezone(&Utc))
        .unwrap_or_else(|_| Utc::now())
}
