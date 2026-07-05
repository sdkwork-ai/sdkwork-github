import type { GithubAppSdkClient, SessionSnapshot } from '@sdkwork/github-pc-core';
import type {
  MilestoneProgress,
  PageResult,
  TrackerComment,
  TrackerIssue,
  TrackerIssueFilters,
  TrackerIssueView,
  TrackerLabel,
  TrackerMilestone,
  TrackerRoadmap,
  TrackerRoadmapItem,
  TrackerRoadmapView,
  VoteResponse,
} from '../types';

const API_PREFIX = '/app/v3/api';
const TRACKER_BASE = `${API_PREFIX}/github/tracker`;

function resolveScope(context: SessionSnapshot['context']): { tenantId: string; organizationId: string } {
  if (!context?.tenantId) {
    throw new Error('tenant context is required');
  }
  return {
    tenantId: context.tenantId,
    organizationId: context.organizationId ?? '',
  };
}

function buildQueryString(params: Record<string, string | number | undefined | null>): string {
  const pairs: string[] = [];
  for (const [key, value] of Object.entries(params)) {
    if (value !== undefined && value !== null && value !== '') {
      pairs.push(`${encodeURIComponent(key)}=${encodeURIComponent(String(value))}`);
    }
  }
  return pairs.join('&');
}

function appendQueryString(path: string, query: string): string {
  if (!query) return path;
  return path.includes('?') ? `${path}&${query}` : `${path}?${query}`;
}

/**
 * Use the SDK's HttpClient (which manages auth tokens and unwraps the `data`
 * envelope automatically) instead of raw fetch.
 */
function http(sdk: GithubAppSdkClient) {
  return sdk.client.http;
}

// ── Issues ───────────────────────────────────────────────
export async function listTrackerIssues(
  sdk: GithubAppSdkClient,
  context: SessionSnapshot['context'],
  filters: TrackerIssueFilters = {},
): Promise<PageResult<TrackerIssueView>> {
  const scope = resolveScope(context);
  const query = buildQueryString({
    tenant_id: scope.tenantId,
    organization_id: scope.organizationId,
    type: filters.type,
    status: filters.status,
    priority: filters.priority,
    milestone_id: filters.milestone_id,
    label_id: filters.label_id,
    q: filters.q,
    sort: filters.sort,
    page: filters.page ?? 1,
    page_size: filters.page_size ?? 20,
  });
  return http(sdk).get<PageResult<TrackerIssueView>>(
    appendQueryString(`${TRACKER_BASE}/issues`, query),
  );
}

export async function getTrackerIssueDetail(
  sdk: GithubAppSdkClient,
  context: SessionSnapshot['context'],
  issueId: string,
): Promise<TrackerIssueView> {
  const scope = resolveScope(context);
  const query = buildQueryString({
    tenant_id: scope.tenantId,
    organization_id: scope.organizationId,
  });
  const res = await http(sdk).get<{ item: TrackerIssueView }>(
    appendQueryString(`${TRACKER_BASE}/issues/${issueId}`, query),
  );
  return res.item;
}

export async function createTrackerIssue(
  sdk: GithubAppSdkClient,
  context: SessionSnapshot['context'],
  data: {
    title: string;
    description: string;
    type: string;
    priority?: string;
    milestone_id?: string;
    label_ids: string[];
  },
): Promise<TrackerIssue> {
  const scope = resolveScope(context);
  const query = buildQueryString({
    tenant_id: scope.tenantId,
    organization_id: scope.organizationId,
  });
  const res = await http(sdk).post<{ item: TrackerIssue }>(
    appendQueryString(`${TRACKER_BASE}/issues`, query),
    data,
  );
  return res.item;
}

export async function updateTrackerIssue(
  sdk: GithubAppSdkClient,
  context: SessionSnapshot['context'],
  issueId: string,
  data: {
    title?: string;
    status?: string;
    priority?: string;
    assignee_id?: string | null;
    milestone_id?: string | null;
    label_ids?: string[];
  },
): Promise<TrackerIssue> {
  const scope = resolveScope(context);
  const query = buildQueryString({
    tenant_id: scope.tenantId,
    organization_id: scope.organizationId,
  });
  const res = await http(sdk).patch<{ item: TrackerIssue }>(
    appendQueryString(`${TRACKER_BASE}/issues/${issueId}`, query),
    data,
  );
  return res.item;
}

// ── Comments ─────────────────────────────────────────────
export async function listTrackerComments(
  sdk: GithubAppSdkClient,
  issueId: string,
  page = 1,
  pageSize = 50,
): Promise<PageResult<TrackerComment>> {
  const query = buildQueryString({ page, page_size: pageSize });
  return http(sdk).get<PageResult<TrackerComment>>(
    appendQueryString(`${TRACKER_BASE}/issues/${issueId}/comments`, query),
  );
}

export async function createTrackerComment(
  sdk: GithubAppSdkClient,
  issueId: string,
  content: string,
): Promise<TrackerComment> {
  const res = await http(sdk).post<{ item: TrackerComment }>(
    `${TRACKER_BASE}/issues/${issueId}/comments`,
    { content },
  );
  return res.item;
}

// ── Votes ────────────────────────────────────────────────
export async function toggleTrackerVote(
  sdk: GithubAppSdkClient,
  issueId: string,
): Promise<VoteResponse> {
  const res = await http(sdk).post<{ item: VoteResponse }>(
    `${TRACKER_BASE}/issues/${issueId}/votes`,
  );
  return res.item;
}

export async function getTrackerVoteStatus(
  sdk: GithubAppSdkClient,
  issueId: string,
): Promise<VoteResponse> {
  const res = await http(sdk).get<{ item: VoteResponse }>(
    `${TRACKER_BASE}/issues/${issueId}/votes/status`,
  );
  return res.item;
}

// ── Labels ───────────────────────────────────────────────
export async function listTrackerLabels(
  sdk: GithubAppSdkClient,
  context: SessionSnapshot['context'],
): Promise<TrackerLabel[]> {
  const scope = resolveScope(context);
  const query = buildQueryString({
    tenant_id: scope.tenantId,
    organization_id: scope.organizationId,
  });
  const res = await http(sdk).get<{ item: { labels: TrackerLabel[] } }>(
    appendQueryString(`${TRACKER_BASE}/labels`, query),
  );
  return res.item?.labels ?? [];
}

export async function createTrackerLabel(
  sdk: GithubAppSdkClient,
  context: SessionSnapshot['context'],
  data: { name: string; color?: string; description?: string },
): Promise<TrackerLabel> {
  const scope = resolveScope(context);
  const query = buildQueryString({
    tenant_id: scope.tenantId,
    organization_id: scope.organizationId,
  });
  const res = await http(sdk).post<{ item: TrackerLabel }>(
    appendQueryString(`${TRACKER_BASE}/labels`, query),
    data,
  );
  return res.item;
}

// ── Milestones ───────────────────────────────────────────
export async function listTrackerMilestones(
  sdk: GithubAppSdkClient,
  context: SessionSnapshot['context'],
): Promise<MilestoneProgress[]> {
  const scope = resolveScope(context);
  const query = buildQueryString({
    tenant_id: scope.tenantId,
    organization_id: scope.organizationId,
  });
  const res = await http(sdk).get<{ item: { milestones: MilestoneProgress[] } }>(
    appendQueryString(`${TRACKER_BASE}/milestones`, query),
  );
  return res.item?.milestones ?? [];
}

export async function createTrackerMilestone(
  sdk: GithubAppSdkClient,
  context: SessionSnapshot['context'],
  data: { title: string; description?: string; due_date?: string },
): Promise<TrackerMilestone> {
  const scope = resolveScope(context);
  const query = buildQueryString({
    tenant_id: scope.tenantId,
    organization_id: scope.organizationId,
  });
  const res = await http(sdk).post<{ item: TrackerMilestone }>(
    appendQueryString(`${TRACKER_BASE}/milestones`, query),
    data,
  );
  return res.item;
}

// ── Roadmaps ─────────────────────────────────────────────
export async function listTrackerRoadmaps(
  sdk: GithubAppSdkClient,
  context: SessionSnapshot['context'],
  page = 1,
  pageSize = 20,
): Promise<PageResult<TrackerRoadmap>> {
  const scope = resolveScope(context);
  const query = buildQueryString({
    tenant_id: scope.tenantId,
    organization_id: scope.organizationId,
    page,
    page_size: pageSize,
  });
  return http(sdk).get<PageResult<TrackerRoadmap>>(
    appendQueryString(`${TRACKER_BASE}/roadmaps`, query),
  );
}

export async function createTrackerRoadmap(
  sdk: GithubAppSdkClient,
  context: SessionSnapshot['context'],
  data: { title: string; description?: string; start_date?: string; target_date?: string },
): Promise<TrackerRoadmap> {
  const scope = resolveScope(context);
  const query = buildQueryString({
    tenant_id: scope.tenantId,
    organization_id: scope.organizationId,
  });
  const res = await http(sdk).post<{ item: TrackerRoadmap }>(
    appendQueryString(`${TRACKER_BASE}/roadmaps`, query),
    data,
  );
  return res.item;
}

export async function getTrackerRoadmapDetail(
  sdk: GithubAppSdkClient,
  context: SessionSnapshot['context'],
  roadmapId: string,
): Promise<TrackerRoadmapView> {
  const scope = resolveScope(context);
  const query = buildQueryString({
    tenant_id: scope.tenantId,
    organization_id: scope.organizationId,
  });
  const res = await http(sdk).get<{ item: TrackerRoadmapView }>(
    appendQueryString(`${TRACKER_BASE}/roadmaps/${roadmapId}`, query),
  );
  return res.item;
}

export async function addTrackerRoadmapItem(
  sdk: GithubAppSdkClient,
  roadmapId: string,
  data: { issue_id: string; track?: string; start_date?: string; target_date?: string; sort_order?: number },
): Promise<TrackerRoadmapItem> {
  const res = await http(sdk).post<{ item: TrackerRoadmapItem }>(
    `${TRACKER_BASE}/roadmaps/${roadmapId}/items`,
    data,
  );
  return res.item;
}

export async function removeTrackerRoadmapItem(
  sdk: GithubAppSdkClient,
  roadmapId: string,
  itemId: string,
): Promise<void> {
  await http(sdk).delete(`${TRACKER_BASE}/roadmaps/${roadmapId}/items/${itemId}`);
}
