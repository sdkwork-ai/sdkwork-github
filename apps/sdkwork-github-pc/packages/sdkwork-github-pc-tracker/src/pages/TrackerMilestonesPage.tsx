import { useCallback, useEffect, useState } from 'react';
import { useGithubPcRuntime } from '@sdkwork/github-pc-core';
import { createTrackerMilestone, listTrackerMilestones } from '../services/trackerService';
import type { MilestoneProgress } from '../types';

export function TrackerMilestonesPage() {
  const { githubSdk, session } = useGithubPcRuntime();
  const [milestones, setMilestones] = useState<MilestoneProgress[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showCreate, setShowCreate] = useState(false);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const list = await listTrackerMilestones(githubSdk, session.getSnapshot().context);
      setMilestones(list ?? []);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, [githubSdk, session]);

  useEffect(() => { void load(); }, [load]);

  const handleCreate = async (data: { title: string; description?: string; due_date?: string }) => {
    try {
      await createTrackerMilestone(githubSdk, session.getSnapshot().context, data);
      setShowCreate(false);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  };

  return (
    <section>
      <header style={{ display: 'flex', gap: '12px', alignItems: 'center', marginBottom: '16px' }}>
        <h2 style={{ margin: 0 }}>Milestones</h2>
        <button type="button" onClick={() => setShowCreate(!showCreate)}>
          {showCreate ? 'Cancel' : 'New Milestone'}
        </button>
      </header>

      {error && <p role="alert" style={{ color: '#d73a4a' }}>{error}</p>}

      {showCreate && (
        <MilestoneCreateForm onSubmit={handleCreate} onCancel={() => setShowCreate(false)} />
      )}

      {loading ? (
        <p>Loading milestones...</p>
      ) : milestones.length === 0 ? (
        <p>No milestones found. Create one to track progress.</p>
      ) : (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '12px' }}>
          {milestones.map((ms) => {
            const pct = ms.total_issues > 0 ? Math.round((ms.closed_issues / ms.total_issues) * 100) : 0;
            return (
              <div key={ms.id} style={{
                border: '1px solid #30363d',
                borderRadius: '6px',
                padding: '12px 16px',
                background: '#0d1117',
              }}>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '8px' }}>
                  <strong>{ms.title}</strong>
                  <span style={{ color: '#8b949e', fontSize: '0.85em' }}>{ms.status}</span>
                </div>
                <div style={{
                  height: '8px',
                  background: '#21262d',
                  borderRadius: '4px',
                  overflow: 'hidden',
                  marginBottom: '4px',
                }}>
                  <div style={{
                    height: '100%',
                    width: `${pct}%`,
                    background: pct === 100 ? '#238636' : '#1f6feb',
                    transition: 'width 0.3s',
                  }} />
                </div>
                <div style={{ display: 'flex', gap: '12px', fontSize: '0.85em', color: '#8b949e' }}>
                  <span>{ms.closed_issues} closed</span>
                  <span>{ms.open_issues} open</span>
                  <span>{ms.total_issues} total</span>
                  <span>{pct}% complete</span>
                  {ms.due_date && <span>Due: {ms.due_date}</span>}
                </div>
              </div>
            );
          })}
        </div>
      )}
    </section>
  );
}

function MilestoneCreateForm({
  onSubmit,
  onCancel,
}: {
  onSubmit: (data: { title: string; description?: string; due_date?: string }) => void;
  onCancel: () => void;
}) {
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [dueDate, setDueDate] = useState('');

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        if (!title.trim()) return;
        onSubmit({
          title: title.trim(),
          description: description.trim() || undefined,
          due_date: dueDate || undefined,
        });
      }}
      style={{
        border: '1px solid #30363d',
        borderRadius: '6px',
        padding: '16px',
        marginBottom: '16px',
        background: '#161b22',
      }}
    >
      <input
        type="text"
        placeholder="Milestone title"
        value={title}
        onChange={(e) => setTitle(e.target.value)}
        style={{ width: '100%', padding: '8px', marginBottom: '8px', background: '#0d1117', border: '1px solid #30363d', borderRadius: '4px', color: '#e6edf3' }}
        required
      />
      <textarea
        placeholder="Description (optional)"
        value={description}
        onChange={(e) => setDescription(e.target.value)}
        rows={2}
        style={{ width: '100%', padding: '8px', marginBottom: '8px', background: '#0d1117', border: '1px solid #30363d', borderRadius: '4px', color: '#e6edf3' }}
      />
      <input
        type="date"
        placeholder="Due date"
        value={dueDate}
        onChange={(e) => setDueDate(e.target.value)}
        style={{ padding: '6px', marginBottom: '8px', background: '#0d1117', border: '1px solid #30363d', borderRadius: '4px', color: '#e6edf3' }}
      />
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
