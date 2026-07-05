import { Navigate, Route, Routes } from 'react-router-dom';
import {
  IntegrationPage,
  IssuesPage,
  PlansPage,
  RepositoriesPage,
  WorkspaceShell,
} from '@sdkwork/github-pc-workspace';
import {
  TrackerIssuesPage,
  TrackerIssueDetailPage,
  TrackerRoadmapPage,
  TrackerMilestonesPage,
} from '@sdkwork/github-pc-tracker';

export function AppShell() {
  return (
    <Routes>
      <Route element={<WorkspaceShell />}>
        <Route index element={<Navigate replace to="/repositories" />} />
        <Route path="repositories" element={<RepositoriesPage />} />
        <Route path="issues" element={<IssuesPage />} />
        <Route path="plans" element={<PlansPage />} />
        <Route path="integration" element={<IntegrationPage />} />
        {/* Tracker routes */}
        <Route path="tracker/issues" element={<TrackerIssuesPage />} />
        <Route path="tracker/issues/:issueId" element={<TrackerIssueDetailPage />} />
        <Route path="tracker/roadmap" element={<TrackerRoadmapPage />} />
        <Route path="tracker/milestones" element={<TrackerMilestonesPage />} />
      </Route>
    </Routes>
  );
}
