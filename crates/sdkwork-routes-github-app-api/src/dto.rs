use serde::{Deserialize, Serialize};

use sdkwork_github_integration_service::domain::{
    IntegrationStatus, LinkIntegrationCommand, SyncResult,
};

#[derive(Debug, Deserialize)]
pub struct PageQuery {
    pub tenant_id: Option<String>,
    pub organization_id: Option<String>,
    pub operator_id: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    pub repository_id: Option<String>,
}

// ── Tracker DTOs ─────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct TrackerIssueListQuery {
    pub tenant_id: Option<String>,
    pub organization_id: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
    #[serde(rename = "type")]
    pub issue_type: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub milestone_id: Option<String>,
    pub label_id: Option<String>,
    pub q: Option<String>,
    pub sort: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateTrackerIssueRequest {
    pub title: String,
    pub description: String,
    #[serde(rename = "type")]
    pub issue_type: String,
    pub priority: Option<String>,
    pub milestone_id: Option<String>,
    pub label_ids: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateTrackerIssueRequest {
    pub title: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub assignee_id: Option<Option<String>>,
    pub milestone_id: Option<Option<String>>,
    pub label_ids: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateTrackerCommentRequest {
    pub content: String,
}

#[derive(Debug, Serialize)]
pub struct VoteResponse {
    pub voted: bool,
}

#[derive(Debug, Serialize)]
pub struct VoteStatusResponse {
    pub voted: bool,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateTrackerLabelRequest {
    pub name: String,
    pub color: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateTrackerMilestoneRequest {
    pub title: String,
    pub description: Option<String>,
    pub due_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct MilestoneIssueListQuery {
    pub tenant_id: Option<String>,
    pub organization_id: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateTrackerRoadmapRequest {
    pub title: String,
    pub description: Option<String>,
    pub start_date: Option<String>,
    pub target_date: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct UpdateTrackerRoadmapRequest {
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub status: Option<String>,
    pub start_date: Option<Option<String>>,
    pub target_date: Option<Option<String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateTrackerRoadmapItemRequest {
pub issue_id: String,
pub track: Option<String>,
pub start_date: Option<String>,
pub target_date: Option<String>,
pub sort_order: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct TrackerLabelsResponse {
    pub labels: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct TrackerMilestonesResponse {
    pub milestones: Vec<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct CommandAcceptedResponse {
    pub accepted: bool,
}

#[derive(Debug, Serialize)]
pub struct SyncResponse {
    pub provider: String,
    pub synced_count: u64,
}

#[derive(Debug, Serialize)]
pub struct CatalogBootstrapResponse {
    pub repositories_synced: u64,
    pub issues_synced: u64,
    pub plans_created: u64,
    pub plan_items_created: u64,
}

impl From<sdkwork_github_integration_service::domain::CatalogBootstrapResult>
    for CatalogBootstrapResponse
{
    fn from(value: sdkwork_github_integration_service::domain::CatalogBootstrapResult) -> Self {
        Self {
            repositories_synced: value.repositories_synced,
            issues_synced: value.issues_synced,
            plans_created: value.plans_created,
            plan_items_created: value.plan_items_created,
        }
    }
}

impl From<SyncResult> for SyncResponse {
    fn from(value: SyncResult) -> Self {
        Self {
            provider: value.provider,
            synced_count: value.synced_count,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct IntegrationStatusResponse {
    pub provider: String,
    pub linked: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_account_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scopes: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_synced_at: Option<String>,
}

impl From<IntegrationStatus> for IntegrationStatusResponse {
    fn from(value: IntegrationStatus) -> Self {
        Self {
            provider: value.provider,
            linked: value.linked,
            status: value.status,
            external_account_id: value.external_account_id,
            scopes: value.scopes,
            last_synced_at: value
                .last_synced_at
                .map(|timestamp| timestamp.to_rfc3339()),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct LinkIntegrationRequest {
    pub access_token: String,
    pub external_account_id: Option<String>,
    pub scopes: Option<String>,
}

impl From<LinkIntegrationRequest> for LinkIntegrationCommand {
    fn from(value: LinkIntegrationRequest) -> Self {
        Self {
            access_token: value.access_token,
            external_account_id: value.external_account_id,
            scopes: value.scopes,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct OAuthBeginResponse {
    pub provider: String,
    pub authorization_url: String,
    pub state: String,
}

impl From<sdkwork_github_integration_service::domain::OAuthBeginResult> for OAuthBeginResponse {
    fn from(value: sdkwork_github_integration_service::domain::OAuthBeginResult) -> Self {
        Self {
            provider: value.provider,
            authorization_url: value.authorization_url,
            state: value.state,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
    pub state: String,
    pub code: String,
}
