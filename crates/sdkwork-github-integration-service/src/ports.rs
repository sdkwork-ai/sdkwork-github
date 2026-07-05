use async_trait::async_trait;

use crate::domain::{
    IntegrationStatus, Issue, LinkIntegrationCommand, Page, Plan, PlanItem, ProviderAccount,
    Repository,
};
use crate::domain::{
    CreateTrackerIssueCommand, CreateTrackerRoadmapItemCommand, TrackerComment, TrackerIssue,
    TrackerIssueQuery, TrackerLabel, TrackerMilestone, TrackerRoadmap,
    TrackerRoadmapItem, UpdateTrackerIssueCommand,
};
use crate::error::ServiceError;

#[async_trait]
pub trait GitHubStore: Send + Sync + Clone {
    async fn list_repositories(
        &self,
        tenant_id: &str,
        organization_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<Page<Repository>, ServiceError>;

    async fn list_issues(
        &self,
        tenant_id: &str,
        organization_id: &str,
        repository_id: Option<&str>,
        page: u32,
        page_size: u32,
    ) -> Result<Page<Issue>, ServiceError>;

    async fn list_plans(
        &self,
        tenant_id: &str,
        organization_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<Page<Plan>, ServiceError>;

    async fn list_plan_items_for_plan_ids(
        &self,
        plan_ids: &[String],
    ) -> Result<Vec<PlanItem>, ServiceError>;
}

#[async_trait]
pub trait GitHubSyncStore: GitHubStore {
    async fn upsert_repository(&self, repository: &Repository) -> Result<(), ServiceError>;

    async fn upsert_issue(&self, issue: &Issue) -> Result<(), ServiceError>;

    async fn upsert_plan(&self, plan: &Plan) -> Result<(), ServiceError>;

    async fn upsert_plan_item(&self, item: &PlanItem) -> Result<(), ServiceError>;

    async fn find_active_provider_account(
        &self,
        tenant_id: &str,
        organization_id: &str,
        provider: &str,
    ) -> Result<Option<ProviderAccount>, ServiceError>;

    async fn upsert_provider_account(
        &self,
        account: &ProviderAccount,
    ) -> Result<(), ServiceError>;

    async fn revoke_provider_account(
        &self,
        tenant_id: &str,
        organization_id: &str,
        provider: &str,
    ) -> Result<(), ServiceError>;

    async fn get_integration_status(
        &self,
        tenant_id: &str,
        organization_id: &str,
        provider: &str,
    ) -> Result<IntegrationStatus, ServiceError>;

    async fn link_integration(
        &self,
        tenant_id: &str,
        organization_id: &str,
        provider: &str,
        command: &LinkIntegrationCommand,
        access_token_cipher: &str,
    ) -> Result<IntegrationStatus, ServiceError>;

    async fn touch_provider_last_synced(
        &self,
        tenant_id: &str,
        organization_id: &str,
        provider: &str,
    ) -> Result<(), ServiceError>;

    async fn create_oauth_pending(
        &self,
        state: &str,
        tenant_id: &str,
        organization_id: &str,
        expires_at: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), ServiceError>;

    async fn consume_oauth_pending(
        &self,
        state: &str,
    ) -> Result<Option<(String, String)>, ServiceError>;

    async fn purge_expired_oauth_pending(&self) -> Result<(), ServiceError>;

    async fn list_admin_integrations(
        &self,
        page: u32,
        page_size: u32,
    ) -> Result<Page<crate::domain::AdminIntegrationView>, ServiceError>;
}

#[async_trait]
pub trait TrackerStore: GitHubStore {
    // ── Issues ──────────────────────────────────────────
    async fn list_tracker_issues(
        &self,
        tenant_id: &str,
        organization_id: &str,
        query: &TrackerIssueQuery,
        page: u32,
        page_size: u32,
    ) -> Result<Page<TrackerIssue>, ServiceError>;

    async fn get_tracker_issue(
        &self,
        tenant_id: &str,
        organization_id: &str,
        issue_id: &str,
    ) -> Result<TrackerIssue, ServiceError>;

    async fn create_tracker_issue(
        &self,
        tenant_id: &str,
        organization_id: &str,
        command: &CreateTrackerIssueCommand,
        submitted_by: &str,
    ) -> Result<TrackerIssue, ServiceError>;

    async fn update_tracker_issue(
        &self,
        tenant_id: &str,
        organization_id: &str,
        issue_id: &str,
        command: &UpdateTrackerIssueCommand,
    ) -> Result<TrackerIssue, ServiceError>;

    async fn list_labels_for_issue_ids(
        &self,
        issue_ids: &[String],
    ) -> Result<Vec<(String, TrackerLabel)>, ServiceError>;

    async fn get_milestone_for_issue(
        &self,
        milestone_id: &str,
    ) -> Result<Option<TrackerMilestone>, ServiceError>;

    // ── Comments ────────────────────────────────────────
    async fn list_tracker_comments(
        &self,
        issue_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<Page<TrackerComment>, ServiceError>;

    async fn create_tracker_comment(
        &self,
        issue_id: &str,
        author_id: &str,
        content: &str,
    ) -> Result<TrackerComment, ServiceError>;

    // ── Votes ───────────────────────────────────────────
    async fn toggle_tracker_vote(
        &self,
        issue_id: &str,
        user_id: &str,
    ) -> Result<bool, ServiceError>;

    async fn has_voted(&self, issue_id: &str, user_id: &str) -> Result<bool, ServiceError>;

    // ── Labels ──────────────────────────────────────────
    async fn list_tracker_labels(
        &self,
        tenant_id: &str,
        organization_id: &str,
    ) -> Result<Vec<TrackerLabel>, ServiceError>;

    async fn create_tracker_label(
        &self,
        tenant_id: &str,
        organization_id: &str,
        name: &str,
        color: &str,
        description: Option<&str>,
    ) -> Result<TrackerLabel, ServiceError>;

    // ── Milestones ──────────────────────────────────────
    async fn list_tracker_milestones(
        &self,
        tenant_id: &str,
        organization_id: &str,
        status: Option<&str>,
    ) -> Result<Vec<crate::domain::MilestoneProgress>, ServiceError>;

    async fn create_tracker_milestone(
        &self,
        tenant_id: &str,
        organization_id: &str,
        title: &str,
        description: Option<&str>,
        due_date: Option<&str>,
    ) -> Result<TrackerMilestone, ServiceError>;

    async fn get_tracker_milestone_issues(
        &self,
        tenant_id: &str,
        organization_id: &str,
        milestone_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<Page<TrackerIssue>, ServiceError>;

    // ── Roadmaps ────────────────────────────────────────
    async fn list_tracker_roadmaps(
        &self,
        tenant_id: &str,
        organization_id: &str,
        page: u32,
        page_size: u32,
    ) -> Result<Page<TrackerRoadmap>, ServiceError>;

    async fn create_tracker_roadmap(
        &self,
        tenant_id: &str,
        organization_id: &str,
        title: &str,
        description: Option<&str>,
        start_date: Option<&str>,
        target_date: Option<&str>,
    ) -> Result<TrackerRoadmap, ServiceError>;

    async fn get_tracker_roadmap(
        &self,
        tenant_id: &str,
        organization_id: &str,
        roadmap_id: &str,
    ) -> Result<TrackerRoadmap, ServiceError>;

    async fn update_tracker_roadmap(
        &self,
        tenant_id: &str,
        organization_id: &str,
        roadmap_id: &str,
        title: Option<&str>,
        description: Option<Option<&str>>,
        status: Option<&str>,
        start_date: Option<Option<&str>>,
        target_date: Option<Option<&str>>,
    ) -> Result<TrackerRoadmap, ServiceError>;

    async fn list_tracker_roadmap_items(
        &self,
        roadmap_id: &str,
    ) -> Result<Vec<TrackerRoadmapItem>, ServiceError>;

    async fn add_tracker_roadmap_item(
        &self,
        roadmap_id: &str,
        command: &CreateTrackerRoadmapItemCommand,
    ) -> Result<TrackerRoadmapItem, ServiceError>;

    async fn remove_tracker_roadmap_item(
        &self,
        roadmap_id: &str,
        item_id: &str,
    ) -> Result<(), ServiceError>;
}
