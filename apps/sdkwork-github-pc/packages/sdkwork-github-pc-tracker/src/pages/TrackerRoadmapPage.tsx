import { useCallback, useEffect, useState } from 'react';
import { useGithubPcRuntime } from '@sdkwork/github-pc-core';
import {
  createTrackerRoadmap,
  getTrackerRoadmapDetail,
  listTrackerRoadmaps,
} from '../services/trackerService';
import type { TrackerRoadmap, TrackerRoadmapView } from '../types';

export function TrackerRoadmapPage() {
  const { githubSdk, session } = useGithubPcRuntime();
  const [roadmaps, setRoadmaps] = useState<TrackerRoadmap[]>([]);
  const [selectedRoadmap, setSelectedRoadmap] = useState<TrackerRoadmapView | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [showCreate, setShowCreate] = useState(false);

  const load = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const page = await listTrackerRoadmaps(githubSdk, session.getSnapshot().context);
      setRoadmaps(page.items ?? []);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, [githubSdk, session]);

  useEffect(() => { void load(); }, [load]);

  const handleSelect = async (id: string) => {
    try {
      const detail = await getTrackerRoadmapDetail(githubSdk, session.getSnapshot().context, id);
      setSelectedRoadmap(detail);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  };

  const handleCreate = async (data: { title: string; description?: string; start_date?: string; target_date?: string }) => {
    try {
      await createTrackerRoadmap(githubSdk, session.getSnapshot().context, data);
      setShowCreate(false);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  };

  if (loading) return <p>Loading roadmaps...</p>;

  if (selectedRoadmap) {
    return (
      <section>
        <button type="button" onClick={() => setSelectedRoadmap(null)} style={{ marginBottom: '12px' }}>
          ← Back to Roadmaps
        </button>
        <h2>{selectedRoadmap.title}</h2>
        {selectedRoadmap.description && <p style={{ color: '#8b949e' }}>{selectedRoadmap.description}</p>}
        <div style={{ display: 'flex', gap: '16px', marginBottom: '16px' }}>
          <span>Status: {selectedRoadmap.status}</span>
          {selectedRoadmap.start_date && <span>Start: {selectedRoadmap.start_date}</span>}
          {selectedRoadmap.target_date && <span>Target: {selectedRoadmap.target_date}</span>}
        </div>
        <h3>Items ({selectedRoadmap.items.length})</h3>
        <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
          {selectedRoadmap.items.map((item) => (
            <div key={item.id} style={{
              border: '1px solid #30363d',
              borderRadius: '6px',
              padding: '8px 12px',
              background: '#161b22',
            }}>
              <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
                <strong>{item.issue.title}</strong>
                <span style={{ color: '#8b949e', fontSize: '0.85em' }}>{item.issue.type} · {item.issue.status}</span>
                {item.track && <span style={{ color: '#8b949e', fontSize: '0.85em' }}>[{item.track}]</span>}
              </div>
              <div style={{ fontSize: '0.85em', color: '#8b949e', marginTop: '4px' }}>
                Priority: {item.issue.priority} · Votes: {item.issue.vote_count}
                {item.start_date && ` · Start: ${item.start_date}`}
                {item.target_date && ` · Target: ${item.target_date}`}
              </div>
            </div>
          ))}
          {selectedRoadmap.items.length === 0 && <p style={{ color: '#8b949e' }}>No items in this roadmap.</p>}
        </div>
      </section>
    );
  }

  return (
    <section>
      <header style={{ display: 'flex', gap: '12px', alignItems: 'center', marginBottom: '16px' }}>
        <h2 style={{ margin: 0 }}>Roadmaps</h2>
        <button type="button" onClick={() => setShowCreate(!showCreate)}>
          {showCreate ? 'Cancel' : 'New Roadmap'}
        </button>
      </header>

      {error && <p role="alert" style={{ color: '#d73a4a' }}>{error}</p>}

      {showCreate && (
        <RoadmapCreateForm onSubmit={handleCreate} onCancel={() => setShowCreate(false)} />
      )}

      {roadmaps.length === 0 ? (
        <p>No roadmaps found. Create one to get started.</p>
      ) : (
        <div style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
          {roadmaps.map((roadmap) => (
            <div
              key={roadmap.id}
              onClick={() => void handleSelect(roadmap.id)}
              style={{
                border: '1px solid #30363d',
                borderRadius: '6px',
                padding: '12px 16px',
                background: '#0d1117',
                cursor: 'pointer',
              }}
            >
              <strong>{roadmap.title}</strong>
              <div style={{ color: '#8b949e', fontSize: '0.85em', marginTop: '4px' }}>
                {roadmap.status}
                {roadmap.start_date && ` · ${roadmap.start_date}`}
                {roadmap.target_date && ` → ${roadmap.target_date}`}
              </div>
              {roadmap.description && (
                <div style={{ color: '#8b949e', marginTop: '4px' }}>{roadmap.description}</div>
              )}
            </div>
          ))}
        </div>
      )}
    </section>
  );
}

function RoadmapCreateForm({
  onSubmit,
  onCancel,
}: {
  onSubmit: (data: { title: string; description?: string; start_date?: string; target_date?: string }) => void;
  onCancel: () => void;
}) {
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [startDate, setStartDate] = useState('');
  const [targetDate, setTargetDate] = useState('');

  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        if (!title.trim()) return;
        onSubmit({
          title: title.trim(),
          description: description.trim() || undefined,
          start_date: startDate || undefined,
          target_date: targetDate || undefined,
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
        placeholder="Roadmap title"
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
      <div style={{ display: 'flex', gap: '8px', marginBottom: '8px' }}>
        <input
          type="date"
          placeholder="Start date"
          value={startDate}
          onChange={(e) => setStartDate(e.target.value)}
          style={{ padding: '6px', background: '#0d1117', border: '1px solid #30363d', borderRadius: '4px', color: '#e6edf3' }}
        />
        <input
          type="date"
          placeholder="Target date"
          value={targetDate}
          onChange={(e) => setTargetDate(e.target.value)}
          style={{ padding: '6px', background: '#0d1117', border: '1px solid #30363d', borderRadius: '4px', color: '#e6edf3' }}
        />
      </div>
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
