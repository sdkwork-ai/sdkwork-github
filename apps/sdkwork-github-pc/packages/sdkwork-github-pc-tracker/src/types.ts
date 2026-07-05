export interface TrackerLabel {
  id: string;
  name: string;
  color: string;
  description?: string;
}

export interface TrackerMilestone {
  id: string;
  title: string;
  description?: string;
  status: string;
  due_date?: string;
}

export interface MilestoneProgress {
  id: string;
  title: string;
  status: string;
  due_date?: string;
  total_issues: number;
  open_issues: number;
  closed_issues: number;
}

export interface TrackerIssue {
  id: string;
  title: string;
  description: string;
  type: string;
  status: string;
  priority: string;
  submitted_by: string;
  assignee_id?: string;
  milestone_id?: string;
  github_issue_id?: string;
  vote_count: number;
  comment_count: number;
  created_at: string;
  updated_at: string;
}

export interface TrackerIssueView extends TrackerIssue {
  labels: TrackerLabel[];
  milestone?: TrackerMilestone;
}

export interface TrackerComment {
  id: string;
  issue_id: string;
  author_id: string;
  content: string;
  created_at: string;
  updated_at: string;
}

export interface TrackerRoadmap {
  id: string;
  title: string;
  description?: string;
  status: string;
  start_date?: string;
  target_date?: string;
}

export interface TrackerRoadmapItem {
  id: string;
  roadmap_id: string;
  issue_id: string;
  track?: string;
  start_date?: string;
  target_date?: string;
  sort_order: number;
}

export interface TrackerRoadmapItemView extends TrackerRoadmapItem {
  issue: TrackerIssue;
  labels: TrackerLabel[];
}

export interface TrackerRoadmapView extends TrackerRoadmap {
  items: TrackerRoadmapItemView[];
}

export interface VoteResponse {
  voted: boolean;
}

export interface PageInfo {
  mode?: string;
  page?: number;
  pageSize?: number;
  totalItems?: string;
  totalPages?: number;
  hasMore?: boolean;
}

export interface PageResult<T> {
  items: T[];
  pageInfo: PageInfo;
}

export interface TrackerIssueFilters {
  type?: string;
  status?: string;
  priority?: string;
  milestone_id?: string;
  label_id?: string;
  q?: string;
  sort?: string;
  page?: number;
  page_size?: number;
}
