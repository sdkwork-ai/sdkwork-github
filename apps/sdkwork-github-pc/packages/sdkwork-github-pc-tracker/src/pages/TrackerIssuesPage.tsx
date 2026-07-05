import { useCallback, useEffect, useState } from 'react';
import { useGithubPcRuntime } from '@sdkwork/github-pc-core';
import { createTrackerIssue, listTrackerIssues, listTrackerLabels } from '../services/trackerService';
import type { TrackerIssueFilters, TrackerIssueView, TrackerLabel } from '../types';

const ISSUE_TYPES = ['bug', 'feature', 'enhancement', 'question', 'task'];
const PRIORITIES = ['low', 'medium', 'high', 'urgent'];
const STATUSES = ['open', 'in_progress', 'resolved', 'closed'];

const typeColors: Record<string, string> = {
  bug: '#d73a4a',
  feature: '#a2eeef',
  enhancement: '#84b6eb',
  question: '#d876e3',
  task: '#0075ca',
};

const priorityColors: Record<string, string> = {
  low: '#6e7681',
  medium: '#fbca04',
  high: '#d93f0b',
  urgent: '#b60205',
};

export function TrackerIssuesPage() {
  const { githubSdk, session } = useGithubPcRuntime();
  const [issues, setIssues] = useState<TrackerIssueView[]>([]);
  const [labels, setLabels] = useState<TrackerLabel[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showCreate, setShowCreate] = useState(false);
  const [filters, setFilters] = useState<TrackerIssueFilters>({ sort: 'newest' });

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const ctx = session.getSnapshot().context;
      const [page, labelList] = await Promise.all([
        listTrackerIssues(githubSdk, ctx, filters),
        listTrackerLabels(githubSdk, ctx),
      ]);
      setIssues(page.items ?? []);
      setLabels(labelList ?? []);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, [githubSdk, session, filters]);

  useEffect(() => { void load(); }, [load]);

  const handleCreate = async (data: { title: string; description: string; type: string; priority: string; label_ids: string[] }) => {
    try {
      await createTrackerIssue(githubSdk, session.getSnapshot().context, data);
      setShowCreate(false);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  };

  return (
    <section>
      <header style={{ display: 'flex', gap: '12px', alignItems: 'center', marginBottom: '16px' }}>
        <h2 style={{ margin: 0 }}>Issues</h2>
        <button type="button" onClick={() => setShowCreate(!showCreate)}>
          {showCreate ? 'Cancel' : 'New Issue'}
        </button>
      </header>

      {error && <p role="alert" style={{ color: '#d73a4a' }}>{error}</p>}

      {showCreate && (
        <IssueCreateForm labels={labels} onSubmit={handleCreate} onCancel={() => setShowCreate(false)} />
      )}

      <div style={{ display: 'flex', gap: '8px', marginBottom: '16px', flexWrap: 'wrap' }}>
        <select value={filters.type ?? ''} onChange={(e) => setFilters({ ...filters, type: e.target.value || undefined })}>
          <option value="">All Types</option>
          {ISSUE_TYPES.map((t) => <option key={t} value={t}>{t}</option>)}
        </select>
        <select value={filters.status ?? ''} onChange={(e) => setFilters({ ...filters, status: e.target.value || undefined })}>
          <option value="">All Status</option>
          {STATUSES.map((s) => <option key={s} value={s}>{s}</option>)}
        </select>
        <select value={filters.priority ?? ''} onChange={(e) => setFilters({ ...filters, priority: e.target.value || undefined })}>
          <option value="">All Priority</option>
          {PRIORITIES.map((p) => <option key={p} value={p}>{p}</option>)}
        </select>
        <select value={filters.sort ?? 'newest'} onChange={(e) => setFilters({ ...filters, sort: e.target.value })}>
          <option value="newest">Newest</option>
          <option value="oldest">Oldest</option>
          <option value="most_voted">Most Voted</option>
          <option value="most_commented">Most Commented</option>
        </select>
        <input
          type="text"
          placeholder="Search..."
          value={filters.q ?? ''}
          onChange={(e) => setFilters({ ...filters, q: e.target.value || undefined })}
          style={{ flexGrow: 1, minWidth: '150px' }}
        />
      </div>

      {loading ? (
        <p>Loading issues...</p>
      ) : issues.length === 0 ? (
        <p>No issues found. Create one to get started.</p>
      ) : (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
          {issues.map((issue) => (
            <IssueCard key={issue.id} issue={issue} />
          ))}
        </div>
      )}
    </section>
  );
}

function IssueCard({ issue }: { issue: TrackerIssueView }) {
  return (
    <div style={{
      border: '1px solid #30363d',
      borderRadius: '6px',
      padding: '12px 16px',
      background: '#0d1117',
    }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: '8px', marginBottom: '4px' }}>
        <span style={{
          display: 'inline-block',
          width: '12px',
          height: '12px',
          borderRadius: '50%',
          background: typeColors[issue.type] ?? '#6e7681',
        }} />
        <strong>{issue.title}</strong>
        <span style={{ color: '#8b949e', fontSize: '0.85em' }}>#{issue.id.slice(-8)}</span>
      </div>
      <div style={{ display: 'flex', gap: '8px', alignItems: 'center', flexWrap: 'wrap' }}>
        <span style={{
          padding: '2px 8px',
          borderRadius: '12px',
          fontSize: '0.8em',
          background: `${priorityColors[issue.priority] ?? '#6e7681'}22`,
          color: priorityColors[issue.priority] ?? '#6e7681',
        }}>{issue.priority}</span>
        <span style={{ color: '#8b949e', fontSize: '0.85em' }}>{issue.status}</span>
        <span style={{ color: '#8b949e', fontSize: '0.85em' }}>▲ {issue.vote_count} votes</span>
        <span style={{ color: '#8b949e', fontSize: '0.85em' }}>💬 {issue.comment_count} comments</span>
        {issue.labels.map((label) => (
          <span key={label.id} style={{
            padding: '2px 8px',
            borderRadius: '12px',
            fontSize: '0.8em',
            background: `#${label.color}22`,
            color: `#${label.color}`,
            border: `1px solid #${label.color}44`,
          }}>{label.name}</span>
        ))}
      </div>
    </div>
  );
}

function IssueCreateForm({
  labels,
  onSubmit,
  onCancel,
}: {
  labels: TrackerLabel[];
  onSubmit: (data: { title: string; description: string; type: string; priority: string; label_ids: string[] }) => void;
  onCancel: () => void;
}) {
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [type, setType] = useState('bug');
  const [priority, setPriority] = useState('medium');
  const [selectedLabels, setSelectedLabels] = useState<string[]>([]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!title.trim() || !description.trim()) return;
    onSubmit({ title: title.trim(), description: description.trim(), type, priority, label_ids: selectedLabels });
  };

  const toggleLabel = (id: string) => {
    setSelectedLabels((prev) => prev.includes(id) ? prev.filter((l) => l !== id) : [...prev, id]);
  };

  return (
    <form onSubmit={handleSubmit} style={{
      border: '1px solid #30363d',
      borderRadius: '6px',
      padding: '16px',
      marginBottom: '16px',
      background: '#161b22',
    }}>
      <div style={{ marginBottom: '8px' }}>
        <input
          type="text"
          placeholder="Title"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          style={{ width: '100%', padding: '8px', background: '#0d1117', border: '1px solid #30363d', borderRadius: '4px', color: '#e6edf3' }}
          required
        />
      </div>
      <div style={{ marginBottom: '8px' }}>
        <textarea
          placeholder="Description"
          value={description}
          onChange={(e) => setDescription(e.target.value)}
          rows={4}
          style={{ width: '100%', padding: '8px', background: '#0d1117', border: '1px solid #30363d', borderRadius: '4px', color: '#e6edf3' }}
          required
        />
      </div>
      <div style={{ display: 'flex', gap: '8px', marginBottom: '8px' }}>
        <select value={type} onChange={(e) => setType(e.target.value)}>
          {ISSUE_TYPES.map((t) => <option key={t} value={t}>{t}</option>)}
        </select>
        <select value={priority} onChange={(e) => setPriority(e.target.value)}>
          {PRIORITIES.map((p) => <option key={p} value={p}>{p}</option>)}
        </select>
      </div>
      {labels.length > 0 && (
        <div style={{ marginBottom: '8px' }}>
          <span style={{ fontSize: '0.85em', color: '#8b949e' }}>Labels:</span>
          <div style={{ display: 'flex', gap: '4px', flexWrap: 'wrap', marginTop: '4px' }}>
            {labels.map((label) => (
              <button
                key={label.id}
                type="button"
                onClick={() => toggleLabel(label.id)}
                style={{
                  padding: '2px 8px',
                  borderRadius: '12px',
                  fontSize: '0.8em',
                  cursor: 'pointer',
                  border: selectedLabels.includes(label.id) ? `2px solid #${label.color}` : '1px solid #30363d',
                  background: selectedLabels.includes(label.id) ? `#${label.color}33` : 'transparent',
                  color: `#${label.color}`,
                }}
              >{label.name}</button>
            ))}
          </div>
        </div>
      )}
      <div style={{ display: 'flex', gap: '8px' }}>
        <button type="submit" style={{ padding: '6px 16px', background: '#238636', color: '#fff', border: 'none', borderRadius: '4px', cursor: 'pointer' }}>
          Create
        </button>
        <button type="button" onClick={onCancel} style={{ padding: '6px 16px', background: '#21262d', color: '#e6edf3', border: '1px solid #30363d', borderRadius: '4px', cursor: 'pointer' }}>
          Cancel
        </button>
      </div>
    </form>
  );
}
