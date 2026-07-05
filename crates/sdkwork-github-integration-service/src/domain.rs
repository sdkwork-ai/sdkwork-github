use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    pub id: String,
    pub tenant_id: String,
    pub organization_id: String,
    pub full_name: String,
    pub owner: String,
    pub description: Option<String>,
    pub default_branch: Option<String>,
    pub html_url: Option<String>,
    pub is_private: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: String,
    pub tenant_id: String,
    pub organization_id: String,
    pub repository_id: String,
    pub number: i64,
    pub title: String,
    pub state: String,
    pub html_url: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: String,
    pub tenant_id: String,
    pub organization_id: String,
    pub repository_id: Option<String>,
    pub title: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanItem {
    pub id: String,
    pub plan_id: String,
    pub title: String,
    pub status: String,
    pub sort_order: i32,
    pub issue_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanView {
    pub id: String,
    pub title: String,
    pub status: String,
    pub repository_id: Option<String>,
    pub items: Vec<PlanItemView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanItemView {
    pub id: String,
    pub title: String,
    pub status: String,
    pub sort_order: i32,
    pub issue_id: Option<String>,
}

impl PlanView {
    pub fn from_plan(plan: Plan, items: Vec<PlanItem>) -> Self {
        Self {
            id: plan.id,
            title: plan.title,
            status: plan.status,
            repository_id: plan.repository_id,
            items: items.into_iter().map(PlanItemView::from).collect(),
        }
    }
}

impl From<PlanItem> for PlanItemView {
    fn from(item: PlanItem) -> Self {
        Self {
            id: item.id,
            title: item.title,
            status: item.status,
            sort_order: item.sort_order,
            issue_id: item.issue_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogBootstrapResult {
    pub repositories_synced: u64,
    pub issues_synced: u64,
    pub plans_created: u64,
    pub plan_items_created: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page<T> {
    pub items: Vec<T>,
    pub page: u32,
    pub page_size: u32,
    pub total: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncResult {
    pub provider: String,
    pub synced_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderAccount {
    pub id: String,
    pub tenant_id: String,
    pub organization_id: String,
    pub provider: String,
    pub external_account_id: Option<String>,
    pub access_token_cipher: String,
    pub scopes: Option<String>,
    pub status: String,
    pub last_synced_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntegrationStatus {
    pub provider: String,
    pub linked: bool,
    pub status: Option<String>,
    pub external_account_id: Option<String>,
    pub scopes: Option<String>,
    pub last_synced_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkIntegrationCommand {
    pub access_token: String,
    pub external_account_id: Option<String>,
    pub scopes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthBeginResult {
    pub provider: String,
    pub authorization_url: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminIntegrationView {
    pub tenant_id: String,
    pub organization_id: String,
    pub provider: String,
    pub linked: bool,
    pub status: Option<String>,
    pub external_account_id: Option<String>,
    pub scopes: Option<String>,
    pub last_synced_at: Option<DateTime<Utc>>,
}

// ════════════════════════════════════════════════════════
// Tracker domain types
// ════════════════════════════════════════════════════════

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerLabel {
    pub id: String,
    pub name: String,
    pub color: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerMilestone {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneProgress {
    pub id: String,
    pub title: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    pub total_issues: u64,
    pub open_issues: u64,
    pub closed_issues: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerIssue {
    pub id: String,
    pub title: String,
    pub description: String,
    #[serde(rename = "type")]
    pub issue_type: String,
    pub status: String,
    pub priority: String,
    pub submitted_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub milestone_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub github_issue_id: Option<String>,
    pub vote_count: i64,
    pub comment_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerIssueView {
    #[serde(flatten)]
    pub issue: TrackerIssue,
    pub labels: Vec<TrackerLabel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub milestone: Option<TrackerMilestone>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerComment {
    pub id: String,
    pub issue_id: String,
    pub author_id: String,
    pub content: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerRoadmap {
    pub id: String,
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerRoadmapItem {
    pub id: String,
    pub roadmap_id: String,
    pub issue_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub track: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_date: Option<String>,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerRoadmapItemView {
    #[serde(flatten)]
    pub item: TrackerRoadmapItem,
    pub issue: TrackerIssue,
    pub labels: Vec<TrackerLabel>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackerRoadmapView {
    #[serde(flatten)]
    pub roadmap: TrackerRoadmap,
    pub items: Vec<TrackerRoadmapItemView>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TrackerIssueQuery {
    pub issue_type: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub milestone_id: Option<String>,
    pub label_id: Option<String>,
    pub q: Option<String>,
    pub sort: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTrackerIssueCommand {
    pub title: String,
    pub description: String,
    pub issue_type: String,
    pub priority: Option<String>,
    pub milestone_id: Option<String>,
    pub label_ids: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UpdateTrackerIssueCommand {
    pub title: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assignee_id: Option<Option<String>>,
    pub milestone_id: Option<Option<String>>,
    pub label_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateTrackerRoadmapItemCommand {
    pub issue_id: String,
    pub track: Option<String>,
    pub start_date: Option<String>,
    pub target_date: Option<String>,
    pub sort_order: Option<i32>,
}
