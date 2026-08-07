use async_trait::async_trait;
use sdkwork_database_config::DatabaseEngine;
use sdkwork_github_integration_service::domain::{
    CreateTrackerIssueCommand, CreateTrackerRoadmapItemCommand, MilestoneProgress, Page,
    TrackerComment, TrackerIssue, TrackerIssueQuery, TrackerLabel, TrackerMilestone,
    TrackerRoadmap, TrackerRoadmapItem, UpdateTrackerIssueCommand,
};
use sdkwork_github_integration_service::error::ServiceError;
use sdkwork_github_integration_service::ports::TrackerStore;

use crate::store::{format_timestamp, parse_ts, SqlGitHubStore};
use chrono::Utc;

fn ph(idx: usize) -> String {
    format!("${idx}")
}

#[async_trait]
impl TrackerStore for SqlGitHubStore {
    async fn list_tracker_issues(
        &self,
        tenant_id: &str,
        organization_id: &str,
        query: &TrackerIssueQuery,
        page: u32,
        page_size: u32,
    ) -> Result<Page<TrackerIssue>, ServiceError> {
        let offset = ((page.saturating_sub(1)) * page_size) as i64;
        let limit = page_size as i64;

        let mut conditions: Vec<String> = Vec::new();
        let mut binds: Vec<String> = Vec::new();
        let mut idx = 0;

        idx += 1;
        conditions.push(format!("tenant_id = {}", ph(idx)));
        binds.push(tenant_id.to_string());
        idx += 1;
        conditions.push(format!("organization_id = {}", ph(idx)));
        binds.push(organization_id.to_string());

        if let Some(ref t) = query.issue_type {
            idx += 1;
            conditions.push(format!("type = {}", ph(idx)));
            binds.push(t.clone());
        }
        if let Some(ref s) = query.status {
            idx += 1;
            conditions.push(format!("status = {}", ph(idx)));
            binds.push(s.clone());
        }
        if let Some(ref p) = query.priority {
            idx += 1;
            conditions.push(format!("priority = {}", ph(idx)));
            binds.push(p.clone());
        }
        if let Some(ref m) = query.milestone_id {
            idx += 1;
            conditions.push(format!("milestone_id = {}", ph(idx)));
            binds.push(m.clone());
        }
        if let Some(ref q) = query.q {
            idx += 1;
            let p = ph(idx);
            conditions.push(format!("title LIKE {p}"));
            binds.push(format!("%{q}%"));
        }

        let where_sql = conditions.join(" AND ");
        let sort = query.sort.as_deref().unwrap_or("newest");
        let order_by = match sort {
            "oldest" => "created_at ASC",
            "most_voted" => "vote_count DESC, created_at DESC",
            "most_commented" => "comment_count DESC, created_at DESC",
            _ => "created_at DESC",
        };

        let cols = "id, title, description, type, status, priority, submitted_by, assignee_id, milestone_id, github_issue_id, vote_count, comment_count, created_at, updated_at";

        idx += 1;
        let limit_p = ph(idx);
        idx += 1;
        let offset_p = ph(idx);

        match self.pool().engine() {
            DatabaseEngine::Sqlite => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            DatabaseEngine::Postgres => {
                let pool = self.pool().as_postgres().expect("postgres pool");
                let count_sql = format!("SELECT COUNT(*) FROM github_tracker_issue WHERE {where_sql}");
                let mut cq = sqlx::query_as::<_, (i64,)>(sqlx::AssertSqlSafe(count_sql));
                for b in &binds { cq = cq.bind(b); }
                let (total,): (i64,) = cq.fetch_one(pool).await.map_err(|e| ServiceError::Repository(e.to_string()))?;

                let select_sql = format!("SELECT {cols} FROM github_tracker_issue WHERE {where_sql} ORDER BY {order_by} LIMIT {limit_p} OFFSET {offset_p}");
                let mut sq = sqlx::query_as::<_, TrackerIssueRow>(sqlx::AssertSqlSafe(select_sql));
                for b in &binds { sq = sq.bind(b); }
                sq = sq.bind(limit).bind(offset);
                let rows = sq.fetch_all(pool).await.map_err(|e| ServiceError::Repository(e.to_string()))?;
                Ok(Page { items: rows.into_iter().map(Into::into).collect(), page, page_size, total: total as u64 })
            }
        }
    }

    async fn get_tracker_issue(
        &self,
        tenant_id: &str,
        organization_id: &str,
        issue_id: &str,
    ) -> Result<TrackerIssue, ServiceError> {
        let cols = "id, title, description, type, status, priority, submitted_by, assignee_id, milestone_id, github_issue_id, vote_count, comment_count, created_at, updated_at";
        match self.pool().engine() {
            DatabaseEngine::Sqlite => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            DatabaseEngine::Postgres => {
                let pool = self.pool().as_postgres().expect("postgres pool");
                let row = sqlx::query_as::<_, TrackerIssueRow>(
                    sqlx::AssertSqlSafe(format!("SELECT {cols} FROM github_tracker_issue WHERE tenant_id = $1 AND organization_id = $2 AND id = $3")),
                )
                .bind(tenant_id).bind(organization_id).bind(issue_id)
                .fetch_optional(pool).await.map_err(|e| ServiceError::Repository(e.to_string()))?;
                row.map(Into::into).ok_or_else(|| ServiceError::NotFound(format!("tracker issue {issue_id} not found")))
            }
        }
    }

    async fn create_tracker_issue(
        &self,
        tenant_id: &str,
        organization_id: &str,
        command: &CreateTrackerIssueCommand,
        submitted_by: &str,
    ) -> Result<TrackerIssue, ServiceError> {
        let id = format!("tracker-issue-{}", uuid::Uuid::new_v4());
        let now = format_timestamp(Utc::now());
        let priority = command.priority.as_deref().unwrap_or("medium");

        let sql = "INSERT INTO github_tracker_issue (id, tenant_id, organization_id, title, description, type, status, priority, submitted_by, assignee_id, milestone_id, github_issue_id, vote_count, comment_count, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, 'open', $7, $8, NULL, $9, NULL, 0, 0, $10, $11)";

        match self.pool().engine() {
            DatabaseEngine::Sqlite => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            DatabaseEngine::Postgres => {
                let pool = self.pool().as_postgres().expect("postgres pool");
                sqlx::query(sql)
                    .bind(&id).bind(tenant_id).bind(organization_id).bind(&command.title).bind(&command.description).bind(&command.issue_type).bind(priority).bind(submitted_by).bind(&command.milestone_id).bind(&now).bind(&now)
                    .execute(pool).await.map_err(|e| ServiceError::Repository(e.to_string()))?;
                for lid in &command.label_ids {
                    sqlx::query("INSERT INTO github_tracker_issue_label (issue_id, label_id) VALUES ($1, $2)")
                        .bind(&id).bind(lid)
                        .execute(pool).await.map_err(|e| ServiceError::Repository(e.to_string()))?;
                }
            }
        }
        self.get_tracker_issue(tenant_id, organization_id, &id).await
    }

    async fn update_tracker_issue(
        &self,
        tenant_id: &str,
        organization_id: &str,
        issue_id: &str,
        command: &UpdateTrackerIssueCommand,
    ) -> Result<TrackerIssue, ServiceError> {
        let now = format_timestamp(Utc::now());

        let mut sets: Vec<String> = Vec::new();
        let mut idx = 0;

        if command.title.is_some() { idx += 1; sets.push(format!("title = {}", ph(idx))); }
        if command.status.is_some() { idx += 1; sets.push(format!("status = {}", ph(idx))); }
        if command.priority.is_some() { idx += 1; sets.push(format!("priority = {}", ph(idx))); }
        if command.assignee_id.is_some() { idx += 1; sets.push(format!("assignee_id = {}", ph(idx))); }
        if command.milestone_id.is_some() { idx += 1; sets.push(format!("milestone_id = {}", ph(idx))); }
        idx += 1; sets.push(format!("updated_at = {}", ph(idx)));
        idx += 1; let id_p = ph(idx);
        idx += 1; let t_p = ph(idx);
        idx += 1; let o_p = ph(idx);

        let sql = format!("UPDATE github_tracker_issue SET {} WHERE id = {} AND tenant_id = {} AND organization_id = {}", sets.join(", "), id_p, t_p, o_p);

        match self.pool().engine() {
            DatabaseEngine::Sqlite => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            DatabaseEngine::Postgres => {
                let pool = self.pool().as_postgres().expect("postgres pool");
                let mut q = sqlx::query(sqlx::AssertSqlSafe(sql));
                if let Some(ref t) = command.title { q = q.bind(t); }
                if let Some(ref s) = command.status { q = q.bind(s); }
                if let Some(ref p) = command.priority { q = q.bind(p); }
                if let Some(ref a) = command.assignee_id { q = q.bind(a.as_deref()); }
                if let Some(ref m) = command.milestone_id { q = q.bind(m.as_deref()); }
                q = q.bind(&now).bind(issue_id).bind(tenant_id).bind(organization_id);
                q.execute(pool).await.map_err(|e| ServiceError::Repository(e.to_string()))?;

                if let Some(ref label_ids) = command.label_ids {
                    sqlx::query("DELETE FROM github_tracker_issue_label WHERE issue_id = $1").bind(issue_id).execute(pool).await.map_err(|e| ServiceError::Repository(e.to_string()))?;
                    for lid in label_ids {
                        sqlx::query("INSERT INTO github_tracker_issue_label (issue_id, label_id) VALUES ($1, $2)").bind(issue_id).bind(lid).execute(pool).await.map_err(|e| ServiceError::Repository(e.to_string()))?;
                    }
                }
            }
        }
        self.get_tracker_issue(tenant_id, organization_id, issue_id).await
    }

    async fn list_labels_for_issue_ids(
        &self,
        issue_ids: &[String],
    ) -> Result<Vec<(String, TrackerLabel)>, ServiceError> {
        if issue_ids.is_empty() { return Ok(Vec::new()); }
        match self.pool().engine() {
            DatabaseEngine::Sqlite => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            DatabaseEngine::Postgres => {
                let pool = self.pool().as_postgres().expect("postgres pool");
                let rows = sqlx::query_as::<_, IssueLabelRow>("SELECT il.issue_id, l.id, l.name, l.color, l.description FROM github_tracker_issue_label il JOIN github_tracker_label l ON il.label_id = l.id WHERE il.issue_id = ANY($1)")
                    .bind(issue_ids).fetch_all(pool).await.map_err(|e| ServiceError::Repository(e.to_string()))?;
                Ok(rows.into_iter().map(Into::into).collect())
            }
        }
    }

    async fn get_milestone_for_issue(&self, milestone_id: &str) -> Result<Option<TrackerMilestone>, ServiceError> {
        match self.pool().engine() {
            DatabaseEngine::Sqlite => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            DatabaseEngine::Postgres => {
                let pool = self.pool().as_postgres().expect("postgres pool");
                let row = sqlx::query_as::<_, MilestoneRow>("SELECT id, title, description, status, due_date FROM github_tracker_milestone WHERE id = $1").bind(milestone_id).fetch_optional(pool).await.map_err(|e| ServiceError::Repository(e.to_string()))?;
                Ok(row.map(Into::into))
            }
        }
    }

    async fn list_tracker_comments(&self, issue_id: &str, page: u32, page_size: u32) -> Result<Page<TrackerComment>, ServiceError> {
        let offset = ((page.saturating_sub(1)) * page_size) as i64;
        let limit = page_size as i64;
        let cols = "id, issue_id, author_id, content, created_at, updated_at";
        match self.pool().engine() {
            DatabaseEngine::Sqlite => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            DatabaseEngine::Postgres => {
                let pool = self.pool().as_postgres().expect("postgres pool");
                let (total,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM github_tracker_comment WHERE issue_id = $1").bind(issue_id).fetch_one(pool).await.map_err(|e| ServiceError::Repository(e.to_string()))?;
                let rows = sqlx::query_as::<_, CommentRow>(sqlx::AssertSqlSafe(format!("SELECT {cols} FROM github_tracker_comment WHERE issue_id = $1 ORDER BY created_at ASC LIMIT $2 OFFSET $3"))).bind(issue_id).bind(limit).bind(offset).fetch_all(pool).await.map_err(|e| ServiceError::Repository(e.to_string()))?;
                Ok(Page { items: rows.into_iter().map(Into::into).collect(), page, page_size, total: total as u64 })
            }
        }
    }

    async fn create_tracker_comment(&self, issue_id: &str, author_id: &str, content: &str) -> Result<TrackerComment, ServiceError> {
        let id = format!("tracker-comment-{}", uuid::Uuid::new_v4());
        let now = format_timestamp(Utc::now());
        let (ins_sql, upd_sql) = ("INSERT INTO github_tracker_comment (id, issue_id, author_id, content, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6)", "UPDATE github_tracker_issue SET comment_count = comment_count + 1, updated_at = $1 WHERE id = $2");
        let cols = "id, issue_id, author_id, content, created_at, updated_at";
        let fetch_sql = format!("SELECT {cols} FROM github_tracker_comment WHERE id = $1");

        match self.pool().engine() {
            DatabaseEngine::Sqlite => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            DatabaseEngine::Postgres => {
                let pool = self.pool().as_postgres().expect("postgres pool");
                sqlx::query(ins_sql).bind(&id).bind(issue_id).bind(author_id).bind(content).bind(&now).bind(&now).execute(pool).await.map_err(|e| ServiceError::Repository(e.to_string()))?;
                sqlx::query(upd_sql).bind(&now).bind(issue_id).execute(pool).await.map_err(|e| ServiceError::Repository(e.to_string()))?;
                let row = sqlx::query_as::<_, CommentRow>(sqlx::AssertSqlSafe(fetch_sql)).bind(&id).fetch_one(pool).await.map_err(|e| ServiceError::Repository(e.to_string()))?;
                Ok(row.into())
            }
        }
    }

    async fn toggle_tracker_vote(&self, issue_id: &str, user_id: &str) -> Result<bool, ServiceError> {
        let now = format_timestamp(Utc::now());
        let already_voted = self.has_voted(issue_id, user_id).await?;
        let (del_sql, dec_sql, ins_sql, inc_sql) = ("DELETE FROM github_tracker_vote WHERE issue_id = $1 AND user_id = $2", "UPDATE github_tracker_issue SET vote_count = GREATEST(vote_count - 1, 0), updated_at = $1 WHERE id = $2", "INSERT INTO github_tracker_vote (id, issue_id, user_id, created_at) VALUES ($1, $2, $3, $4)", "UPDATE github_tracker_issue SET vote_count = vote_count + 1, updated_at = $1 WHERE id = $2");

        match self.pool().engine() {
            DatabaseEngine::Sqlite => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            DatabaseEngine::Postgres => {
                let pool = self.pool().as_postgres().expect("postgres pool");
                if already_voted {
                    sqlx::query(del_sql).bind(issue_id).bind(user_id).execute(pool).await.map_err(|e| ServiceError::Repository(e.to_string()))?;
                    sqlx::query(dec_sql).bind(&now).bind(issue_id).execute(pool).await.map_err(|e| ServiceError::Repository(e.to_string()))?;
                    Ok(false)
                } else {
                    let vote_id = format!("tracker-vote-{}", uuid::Uuid::new_v4());
                    sqlx::query(ins_sql).bind(&vote_id).bind(issue_id).bind(user_id).bind(&now).execute(pool).await.map_err(|e| ServiceError::Repository(e.to_string()))?;
                    sqlx::query(inc_sql).bind(&now).bind(issue_id).execute(pool).await.map_err(|e| ServiceError::Repository(e.to_string()))?;
                    Ok(true)
                }
            }
        }
    }

    async fn has_voted(&self, issue_id: &str, user_id: &str) -> Result<bool, ServiceError> {
        let sql = "SELECT COUNT(*) FROM github_tracker_vote WHERE issue_id = $1 AND user_id = $2";
        let (count,): (i64,) = sqlx::query_as(sql)
            .bind(issue_id)
            .bind(user_id)
            .fetch_one(self.pool().as_postgres().expect("postgres pool"))
            .await
            .map_err(|e| ServiceError::Repository(e.to_string()))?;
        Ok(count > 0)
    }

    async fn list_tracker_labels(&self, tenant_id: &str, organization_id: &str) -> Result<Vec<TrackerLabel>, ServiceError> {
        match self.pool().engine() {
            DatabaseEngine::Sqlite => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            DatabaseEngine::Postgres => {
                let pool = self.pool().as_postgres().expect("postgres pool");
                let rows = sqlx::query_as::<_, LabelRow>("SELECT id, name, color, description FROM github_tracker_label WHERE tenant_id = $1 AND organization_id = $2 ORDER BY name ASC").bind(tenant_id).bind(organization_id).fetch_all(pool).await.map_err(|e| ServiceError::Repository(e.to_string()))?;
                Ok(rows.into_iter().map(Into::into).collect())
            }
        }
    }

    async fn create_tracker_label(&self, tenant_id: &str, organization_id: &str, name: &str, color: &str, description: Option<&str>) -> Result<TrackerLabel, ServiceError> {
        let id = format!("tracker-label-{}", uuid::Uuid::new_v4());
        let now = format_timestamp(Utc::now());
        let sql = "INSERT INTO github_tracker_label (id, tenant_id, organization_id, name, color, description, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)";
        match self.pool().engine() {
            DatabaseEngine::Sqlite => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            DatabaseEngine::Postgres => { let pool = self.pool().as_postgres().expect("postgres pool"); sqlx::query(sql).bind(&id).bind(tenant_id).bind(organization_id).bind(name).bind(color).bind(description).bind(&now).bind(&now).execute(pool).await.map_err(|e| ServiceError::Repository(e.to_string()))?; }
        }
        Ok(TrackerLabel { id, name: name.to_string(), color: color.to_string(), description: description.map(|s| s.to_string()) })
    }

    async fn list_tracker_milestones(&self, tenant_id: &str, organization_id: &str, status: Option<&str>) -> Result<Vec<MilestoneProgress>, ServiceError> {
        let (where_sql, binds): (String, Vec<&str>) = match status {
            Some(s) => { let mut idx = 0; idx += 1; let p1 = ph(idx); idx += 1; let p2 = ph(idx); idx += 1; let p3 = ph(idx); (format!("m.tenant_id = {p1} AND m.organization_id = {p2} AND m.status = {p3}"), vec![tenant_id, organization_id, s]) }
            None => { let mut idx = 0; idx += 1; let p1 = ph(idx); idx += 1; let p2 = ph(idx); (format!("m.tenant_id = {p1} AND m.organization_id = {p2}"), vec![tenant_id, organization_id]) }
        };
        let sql = format!(
            "SELECT m.id, m.title, m.status, m.due_date, COUNT(i.id) as total_issues, SUM(CASE WHEN i.status IN ('open', 'in_progress') THEN 1 ELSE 0 END) as open_issues, SUM(CASE WHEN i.status IN ('resolved', 'closed') THEN 1 ELSE 0 END) as closed_issues
             FROM github_tracker_milestone m LEFT JOIN github_tracker_issue i ON i.milestone_id = m.id AND i.tenant_id = m.tenant_id AND i.organization_id = m.organization_id
             WHERE {where_sql} GROUP BY m.id ORDER BY m.due_date ASC NULLS LAST"
        );
        match self.pool().engine() {
            DatabaseEngine::Sqlite => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            DatabaseEngine::Postgres => {
                let pool = self.pool().as_postgres().expect("postgres pool");
                let mut q = sqlx::query_as::<_, MilestoneProgressRow>(sqlx::AssertSqlSafe(sql));
                for b in &binds { q = q.bind(b); }
                let rows = q.fetch_all(pool).await.map_err(|e| ServiceError::Repository(e.to_string()))?;
                Ok(rows.into_iter().map(Into::into).collect())
            }
        }
    }

    async fn create_tracker_milestone(&self, tenant_id: &str, organization_id: &str, title: &str, description: Option<&str>, due_date: Option<&str>) -> Result<TrackerMilestone, ServiceError> {
        let id = format!("tracker-milestone-{}", uuid::Uuid::new_v4());
        let now = format_timestamp(Utc::now());
        let sql = "INSERT INTO github_tracker_milestone (id, tenant_id, organization_id, title, description, status, due_date, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'open', $6, $7, $8)";
        match self.pool().engine() {
            DatabaseEngine::Sqlite => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            DatabaseEngine::Postgres => { let pool = self.pool().as_postgres().expect("postgres pool"); sqlx::query(sql).bind(&id).bind(tenant_id).bind(organization_id).bind(title).bind(description).bind(due_date).bind(&now).bind(&now).execute(pool).await.map_err(|e| ServiceError::Repository(e.to_string()))?; }
        }
        Ok(TrackerMilestone { id, title: title.to_string(), description: description.map(|s| s.to_string()), status: "open".to_string(), due_date: due_date.map(|s| s.to_string()) })
    }

    async fn get_tracker_milestone_issues(&self, tenant_id: &str, organization_id: &str, milestone_id: &str, page: u32, page_size: u32) -> Result<Page<TrackerIssue>, ServiceError> {
        let offset = ((page.saturating_sub(1)) * page_size) as i64;
        let limit = page_size as i64;
        let cols = "id, title, description, type, status, priority, submitted_by, assignee_id, milestone_id, github_issue_id, vote_count, comment_count, created_at, updated_at";
        match self.pool().engine() {
            DatabaseEngine::Sqlite => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            DatabaseEngine::Postgres => {
                let pool = self.pool().as_postgres().expect("postgres pool");
                let (total,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM github_tracker_issue WHERE tenant_id = $1 AND organization_id = $2 AND milestone_id = $3").bind(tenant_id).bind(organization_id).bind(milestone_id).fetch_one(pool).await.map_err(|e| ServiceError::Repository(e.to_string()))?;
                let rows = sqlx::query_as::<_, TrackerIssueRow>(sqlx::AssertSqlSafe(format!("SELECT {cols} FROM github_tracker_issue WHERE tenant_id = $1 AND organization_id = $2 AND milestone_id = $3 ORDER BY created_at DESC LIMIT $4 OFFSET $5"))).bind(tenant_id).bind(organization_id).bind(milestone_id).bind(limit).bind(offset).fetch_all(pool).await.map_err(|e| ServiceError::Repository(e.to_string()))?;
                Ok(Page { items: rows.into_iter().map(Into::into).collect(), page, page_size, total: total as u64 })
            }
        }
    }

    async fn list_tracker_roadmaps(&self, tenant_id: &str, organization_id: &str, page: u32, page_size: u32) -> Result<Page<TrackerRoadmap>, ServiceError> {
        let offset = ((page.saturating_sub(1)) * page_size) as i64;
        let limit = page_size as i64;
        let cols = "id, title, description, status, start_date, target_date";
        match self.pool().engine() {
            DatabaseEngine::Sqlite => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            DatabaseEngine::Postgres => {
                let pool = self.pool().as_postgres().expect("postgres pool");
                let (total,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM github_tracker_roadmap WHERE tenant_id = $1 AND organization_id = $2").bind(tenant_id).bind(organization_id).fetch_one(pool).await.map_err(|e| ServiceError::Repository(e.to_string()))?;
                let rows = sqlx::query_as::<_, RoadmapRow>(sqlx::AssertSqlSafe(format!("SELECT {cols} FROM github_tracker_roadmap WHERE tenant_id = $1 AND organization_id = $2 ORDER BY created_at DESC LIMIT $3 OFFSET $4"))).bind(tenant_id).bind(organization_id).bind(limit).bind(offset).fetch_all(pool).await.map_err(|e| ServiceError::Repository(e.to_string()))?;
                Ok(Page { items: rows.into_iter().map(Into::into).collect(), page, page_size, total: total as u64 })
            }
        }
    }

    async fn create_tracker_roadmap(&self, tenant_id: &str, organization_id: &str, title: &str, description: Option<&str>, start_date: Option<&str>, target_date: Option<&str>) -> Result<TrackerRoadmap, ServiceError> {
        let id = format!("tracker-roadmap-{}", uuid::Uuid::new_v4());
        let now = format_timestamp(Utc::now());
        let sql = "INSERT INTO github_tracker_roadmap (id, tenant_id, organization_id, title, description, status, start_date, target_date, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, 'active', $6, $7, $8, $9)";
        match self.pool().engine() {
            DatabaseEngine::Sqlite => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            DatabaseEngine::Postgres => { let pool = self.pool().as_postgres().expect("postgres pool"); sqlx::query(sql).bind(&id).bind(tenant_id).bind(organization_id).bind(title).bind(description).bind(start_date).bind(target_date).bind(&now).bind(&now).execute(pool).await.map_err(|e| ServiceError::Repository(e.to_string()))?; }
        }
        Ok(TrackerRoadmap { id, title: title.to_string(), description: description.map(|s| s.to_string()), status: "active".to_string(), start_date: start_date.map(|s| s.to_string()), target_date: target_date.map(|s| s.to_string()) })
    }

    async fn get_tracker_roadmap(&self, tenant_id: &str, organization_id: &str, roadmap_id: &str) -> Result<TrackerRoadmap, ServiceError> {
        let cols = "id, title, description, status, start_date, target_date";
        match self.pool().engine() {
            DatabaseEngine::Sqlite => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            DatabaseEngine::Postgres => {
                let pool = self.pool().as_postgres().expect("postgres pool");
                let row = sqlx::query_as::<_, RoadmapRow>(sqlx::AssertSqlSafe(format!("SELECT {cols} FROM github_tracker_roadmap WHERE tenant_id = $1 AND organization_id = $2 AND id = $3"))).bind(tenant_id).bind(organization_id).bind(roadmap_id).fetch_optional(pool).await.map_err(|e| ServiceError::Repository(e.to_string()))?;
                row.map(Into::into).ok_or_else(|| ServiceError::NotFound(format!("roadmap {roadmap_id} not found")))
            }
        }
    }

    async fn update_tracker_roadmap(&self, tenant_id: &str, organization_id: &str, roadmap_id: &str, title: Option<&str>, description: Option<Option<&str>>, status: Option<&str>, start_date: Option<Option<&str>>, target_date: Option<Option<&str>>) -> Result<TrackerRoadmap, ServiceError> {
        let now = format_timestamp(Utc::now());
        let mut sets: Vec<String> = Vec::new();
        let mut idx = 0;
        if title.is_some() { idx += 1; sets.push(format!("title = {}", ph(idx))); }
        if description.is_some() { idx += 1; sets.push(format!("description = {}", ph(idx))); }
        if status.is_some() { idx += 1; sets.push(format!("status = {}", ph(idx))); }
        if start_date.is_some() { idx += 1; sets.push(format!("start_date = {}", ph(idx))); }
        if target_date.is_some() { idx += 1; sets.push(format!("target_date = {}", ph(idx))); }
        idx += 1; sets.push(format!("updated_at = {}", ph(idx)));
        idx += 1; let id_p = ph(idx);
        idx += 1; let t_p = ph(idx);
        idx += 1; let o_p = ph(idx);
        if sets.is_empty() { return self.get_tracker_roadmap(tenant_id, organization_id, roadmap_id).await; }
        let sql = format!("UPDATE github_tracker_roadmap SET {} WHERE id = {} AND tenant_id = {} AND organization_id = {}", sets.join(", "), id_p, t_p, o_p);

        match self.pool().engine() {
            DatabaseEngine::Sqlite => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            DatabaseEngine::Postgres => {
                let pool = self.pool().as_postgres().expect("postgres pool");
                let mut q = sqlx::query(sqlx::AssertSqlSafe(sql));
                if let Some(t) = title { q = q.bind(t); }
                if let Some(d) = description { q = q.bind(d); }
                if let Some(s) = status { q = q.bind(s); }
                if let Some(sd) = start_date { q = q.bind(sd); }
                if let Some(td) = target_date { q = q.bind(td); }
                q = q.bind(&now).bind(roadmap_id).bind(tenant_id).bind(organization_id);
                q.execute(pool).await.map_err(|e| ServiceError::Repository(e.to_string()))?;
            }
        }
        self.get_tracker_roadmap(tenant_id, organization_id, roadmap_id).await
    }

    async fn list_tracker_roadmap_items(&self, roadmap_id: &str) -> Result<Vec<TrackerRoadmapItem>, ServiceError> {
        let cols = "id, roadmap_id, issue_id, track, start_date, target_date, sort_order";
        match self.pool().engine() {
            DatabaseEngine::Sqlite => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            DatabaseEngine::Postgres => {
                let pool = self.pool().as_postgres().expect("postgres pool");
                let rows = sqlx::query_as::<_, RoadmapItemRow>(sqlx::AssertSqlSafe(format!("SELECT {cols} FROM github_tracker_roadmap_item WHERE roadmap_id = $1 ORDER BY sort_order ASC"))).bind(roadmap_id).fetch_all(pool).await.map_err(|e| ServiceError::Repository(e.to_string()))?;
                Ok(rows.into_iter().map(Into::into).collect())
            }
        }
    }

    async fn add_tracker_roadmap_item(&self, roadmap_id: &str, command: &CreateTrackerRoadmapItemCommand) -> Result<TrackerRoadmapItem, ServiceError> {
        let id = format!("tracker-roadmap-item-{}", uuid::Uuid::new_v4());
        let now = format_timestamp(Utc::now());
        let sort_order = command.sort_order.unwrap_or(0);
        let sql = "INSERT INTO github_tracker_roadmap_item (id, roadmap_id, issue_id, track, start_date, target_date, sort_order, created_at, updated_at) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)";
        match self.pool().engine() {
            DatabaseEngine::Sqlite => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            DatabaseEngine::Postgres => { let pool = self.pool().as_postgres().expect("postgres pool"); sqlx::query(sql).bind(&id).bind(roadmap_id).bind(&command.issue_id).bind(&command.track).bind(&command.start_date).bind(&command.target_date).bind(sort_order).bind(&now).bind(&now).execute(pool).await.map_err(|e| ServiceError::Repository(e.to_string()))?; }
        }
        Ok(TrackerRoadmapItem { id, roadmap_id: roadmap_id.to_string(), issue_id: command.issue_id.clone(), track: command.track.clone(), start_date: command.start_date.clone(), target_date: command.target_date.clone(), sort_order })
    }

    async fn remove_tracker_roadmap_item(&self, _roadmap_id: &str, item_id: &str) -> Result<(), ServiceError> {
        let sql = "DELETE FROM github_tracker_roadmap_item WHERE id = $1";
        match self.pool().engine() {
            DatabaseEngine::Sqlite => {
                return Err(ServiceError::Repository(
                    "github integration store requires a PostgreSQL pool (DATABASE_SPEC: authoritative-server persistence is PostgreSQL only)".to_string(),
                ));
            }
            DatabaseEngine::Postgres => { let pool = self.pool().as_postgres().expect("postgres pool"); sqlx::query(sql).bind(item_id).execute(pool).await.map_err(|e| ServiceError::Repository(e.to_string()))?; }
        }
        Ok(())
    }
}

// ── Row types ────────────────────────────────────────────

#[derive(sqlx::FromRow)]
struct TrackerIssueRow {
    id: String, title: String, description: String,
    #[sqlx(rename = "type")]
    issue_type: String, status: String, priority: String, submitted_by: String,
    assignee_id: Option<String>, milestone_id: Option<String>, github_issue_id: Option<String>,
    vote_count: i64, comment_count: i64, created_at: String, updated_at: String,
}
impl From<TrackerIssueRow> for TrackerIssue {
    fn from(r: TrackerIssueRow) -> Self {
        Self { id: r.id, title: r.title, description: r.description, issue_type: r.issue_type, status: r.status, priority: r.priority, submitted_by: r.submitted_by, assignee_id: r.assignee_id, milestone_id: r.milestone_id, github_issue_id: r.github_issue_id, vote_count: r.vote_count, comment_count: r.comment_count, created_at: parse_ts(&r.created_at), updated_at: parse_ts(&r.updated_at) }
    }
}

#[derive(sqlx::FromRow)]
struct LabelRow { id: String, name: String, color: String, description: Option<String> }
impl From<LabelRow> for TrackerLabel {
    fn from(r: LabelRow) -> Self { Self { id: r.id, name: r.name, color: r.color, description: r.description } }
}

#[derive(sqlx::FromRow)]
struct IssueLabelRow { issue_id: String, id: String, name: String, color: String, description: Option<String> }
impl From<IssueLabelRow> for (String, TrackerLabel) {
    fn from(r: IssueLabelRow) -> Self { (r.issue_id, TrackerLabel { id: r.id, name: r.name, color: r.color, description: r.description }) }
}

#[derive(sqlx::FromRow)]
struct MilestoneRow { id: String, title: String, description: Option<String>, status: String, due_date: Option<String> }
impl From<MilestoneRow> for TrackerMilestone {
    fn from(r: MilestoneRow) -> Self { Self { id: r.id, title: r.title, description: r.description, status: r.status, due_date: r.due_date } }
}

#[derive(sqlx::FromRow)]
struct MilestoneProgressRow { id: String, title: String, status: String, due_date: Option<String>, total_issues: i64, open_issues: Option<i64>, closed_issues: Option<i64> }
impl From<MilestoneProgressRow> for MilestoneProgress {
    fn from(r: MilestoneProgressRow) -> Self { Self { id: r.id, title: r.title, status: r.status, due_date: r.due_date, total_issues: r.total_issues as u64, open_issues: r.open_issues.unwrap_or(0) as u64, closed_issues: r.closed_issues.unwrap_or(0) as u64 } }
}

#[derive(sqlx::FromRow)]
struct CommentRow { id: String, issue_id: String, author_id: String, content: String, created_at: String, updated_at: String }
impl From<CommentRow> for TrackerComment {
    fn from(r: CommentRow) -> Self { Self { id: r.id, issue_id: r.issue_id, author_id: r.author_id, content: r.content, created_at: parse_ts(&r.created_at), updated_at: parse_ts(&r.updated_at) } }
}

#[derive(sqlx::FromRow)]
struct RoadmapRow { id: String, title: String, description: Option<String>, status: String, start_date: Option<String>, target_date: Option<String> }
impl From<RoadmapRow> for TrackerRoadmap {
    fn from(r: RoadmapRow) -> Self { Self { id: r.id, title: r.title, description: r.description, status: r.status, start_date: r.start_date, target_date: r.target_date } }
}

#[derive(sqlx::FromRow)]
struct RoadmapItemRow { id: String, roadmap_id: String, issue_id: String, track: Option<String>, start_date: Option<String>, target_date: Option<String>, sort_order: i32 }
impl From<RoadmapItemRow> for TrackerRoadmapItem {
    fn from(r: RoadmapItemRow) -> Self { Self { id: r.id, roadmap_id: r.roadmap_id, issue_id: r.issue_id, track: r.track, start_date: r.start_date, target_date: r.target_date, sort_order: r.sort_order } }
}
