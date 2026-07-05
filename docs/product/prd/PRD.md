# SDKWork GitHub PRD

Status: active
Owner: SDKWork maintainers
Application: sdkwork-github
Updated: 2026-07-02
Specs: REQUIREMENTS_SPEC.md, DOCUMENTATION_SPEC.md

## Document Map

- Add `PRD-<topic>.md` shards in this directory when the PRD grows beyond one reviewable screen.

## 1. Background And Problem

SDKWork GitHub provides a unified surface for browsing GitHub repositories, issues, and plans, plus a built-in issue tracker that lets users submit bug reports, feature requests, and feedback directly within the SDKWork platform. The tracker module consolidates Issues, Roadmaps, and Milestones into a single cohesive domain package (`sdkwork-github-pc-tracker`), following GitHub's issue-tracking model while remaining SDKWork-native.

## 2. Target Users

- **SDKWork platform users**: submit issues, feature requests, and feedback through the tracker.
- **Project maintainers**: manage roadmaps, milestones, and triage issues via the PC application.
- **Integration administrators**: link GitHub OAuth/PAT credentials for repository and issue sync.

## 3. Goals And Non-Goals

### Goals

- Users can create, view, filter, and update tracker issues (bug, feature, enhancement, question, task).
- Users can vote and comment on issues to express priority and provide context.
- Project maintainers can create and manage milestones with progress tracking.
- Project maintainers can create roadmaps and organize issues into roadmap tracks.
- All tracker data follows the SDKWork API response envelope standard (`SdkWorkApiResponse`).

### Non-Goals

- Full GitHub Issues bi-directional sync (tracker issues are SDKWork-native; GitHub sync covers repository and issue catalog only).
- Kanban board view (roadmap provides timeline visualization; board view is a future enhancement).

## 4. Scope

### In Scope

- Tracker domain: Issues, Labels, Milestones, Roadmaps, Roadmap Items, Comments, Votes.
- API surface: `/app/v3/api/github/tracker/*` endpoints under the `app-api` surface.
- Frontend: `sdkwork-github-pc-tracker` package with Issues, Issue Detail, Roadmaps, and Milestones pages.
- Database: `github_tracker_*` tables (SQLite and PostgreSQL).
- Cross-database compatibility via conditional SQL placeholders.

### Out of Scope

- Email notifications for issue updates.
- Webhook-based GitHub issue mirroring into tracker.

## 5. User Scenarios

1. A user opens the PC application, navigates to Issues, filters by type=feature, and submits a feature request with a label.
2. A maintainer views the issue detail page, votes on it, and adds a comment.
3. A maintainer creates a milestone with a due date, then assigns issues to it.
4. A maintainer creates a roadmap, adds issues as roadmap items with tracks and dates.
5. Users search issues by keyword and sort by most voted or most commented.

## 6. Success Metrics

- Issue creation to resolution cycle time.
- User engagement: vote count and comment density per issue.
- Milestone completion rate (closed vs. open issues).

## 7. Phases

- **Phase 1 (Current)**: Core tracker functionality — issues, labels, milestones, roadmaps, comments, votes. Backend API, database schema, and frontend pages.
- **Phase 2 (Future)**: Roadmap drag-and-drop reordering, milestone burndown charts, issue assignment workflows.

## 8. Linked Requirements

- API response envelope: `API_SPEC.md` section 4.5, sections 14-16.
- SDK generation: `SDK_SPEC.md` section 4.2.
- Frontend service pattern: `FRONTEND_SPEC.md`.

## 9. Open Questions

- Should tracker issues support custom fields beyond the unified issue model?
- Should votes be weighted by user role?
