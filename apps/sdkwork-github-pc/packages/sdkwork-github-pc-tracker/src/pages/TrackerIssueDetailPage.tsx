import { useCallback, useEffect, useState } from 'react';
import { useParams } from 'react-router-dom';
import { useGithubPcRuntime } from '@sdkwork/github-pc-core';
import {
  createTrackerComment,
  getTrackerIssueDetail,
  getTrackerVoteStatus,
  listTrackerComments,
  toggleTrackerVote,
} from '../services/trackerService';
import type { TrackerComment, TrackerIssueView } from '../types';

export function TrackerIssueDetailPage() {
  const { issueId } = useParams<{ issueId: string }>();
  const { githubSdk, session } = useGithubPcRuntime();
  const [issue, setIssue] = useState<TrackerIssueView | null>(null);
  const [comments, setComments] = useState<TrackerComment[]>([]);
  const [voted, setVoted] = useState(false);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [newComment, setNewComment] = useState('');
  const [submitting, setSubmitting] = useState(false);

  const load = useCallback(async () => {
    if (!issueId) return;
    setLoading(true);
    setError(null);
    try {
      const ctx = session.getSnapshot().context;
      const [detail, commentPage, voteStatus] = await Promise.all([
        getTrackerIssueDetail(githubSdk, ctx, issueId),
        listTrackerComments(githubSdk, issueId),
        getTrackerVoteStatus(githubSdk, issueId),
      ]);
      setIssue(detail);
      setComments(commentPage.items ?? []);
      setVoted(voteStatus.voted);
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  }, [githubSdk, session, issueId]);

  useEffect(() => { void load(); }, [load]);

  const handleVote = async () => {
    try {
      const result = await toggleTrackerVote(githubSdk, issueId!);
      setVoted(result.voted);
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    }
  };

  const handleComment = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newComment.trim()) return;
    setSubmitting(true);
    try {
      await createTrackerComment(githubSdk, issueId!, newComment.trim());
      setNewComment('');
      await load();
    } catch (e) {
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setSubmitting(false);
    }
  };

  if (loading) return <p>Loading issue...</p>;
  if (error) return <p role="alert" style={{ color: '#d73a4a' }}>{error}</p>;
  if (!issue) return <p>Issue not found.</p>;

  return (
    <section>
      <div style={{ marginBottom: '16px' }}>
        <h2 style={{ margin: '0 0 8px' }}>{issue.title}</h2>
        <div style={{ display: 'flex', gap: '8px', alignItems: 'center', flexWrap: 'wrap' }}>
          <span style={{ padding: '2px 8px', borderRadius: '12px', fontSize: '0.8em', background: '#21262d' }}>{issue.type}</span>
          <span style={{ padding: '2px 8px', borderRadius: '12px', fontSize: '0.8em', background: '#21262d' }}>{issue.status}</span>
          <span style={{ padding: '2px 8px', borderRadius: '12px', fontSize: '0.8em', background: '#21262d' }}>{issue.priority}</span>
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

      <div style={{
        border: '1px solid #30363d',
        borderRadius: '6px',
        padding: '12px 16px',
        marginBottom: '16px',
        background: '#0d1117',
        whiteSpace: 'pre-wrap',
      }}>{issue.description}</div>

      <div style={{ display: 'flex', gap: '8px', marginBottom: '24px' }}>
        <button
          type="button"
          onClick={() => void handleVote()}
          style={{
            padding: '6px 16px',
            background: voted ? '#238636' : '#21262d',
            color: '#e6edf3',
            border: '1px solid #30363d',
            borderRadius: '4px',
            cursor: 'pointer',
          }}
        >
          {voted ? '▲ Voted' : '△ Vote'} ({issue.vote_count})
        </button>
      </div>

      <h3 style={{ marginBottom: '8px' }}>Comments ({issue.comment_count})</h3>
      <div style={{ display: 'flex', flexDirection: 'column', gap: '8px', marginBottom: '16px' }}>
        {comments.map((comment) => (
          <div key={comment.id} style={{
            border: '1px solid #30363d',
            borderRadius: '6px',
            padding: '8px 12px',
            background: '#161b22',
          }}>
            <div style={{ fontSize: '0.85em', color: '#8b949e', marginBottom: '4px' }}>
              {comment.author_id} · {new Date(comment.created_at).toLocaleString()}
            </div>
            <div style={{ whiteSpace: 'pre-wrap' }}>{comment.content}</div>
          </div>
        ))}
        {comments.length === 0 && <p style={{ color: '#8b949e' }}>No comments yet.</p>}
      </div>

      <form onSubmit={handleComment}>
        <textarea
          placeholder="Write a comment..."
          value={newComment}
          onChange={(e) => setNewComment(e.target.value)}
          rows={3}
          style={{
            width: '100%',
            padding: '8px',
            background: '#0d1117',
            border: '1px solid #30363d',
            borderRadius: '4px',
            color: '#e6edf3',
            marginBottom: '8px',
          }}
        />
        <button
          type="submit"
          disabled={submitting || !newComment.trim()}
          style={{
            padding: '6px 16px',
            background: '#238636',
            color: '#fff',
            border: 'none',
            borderRadius: '4px',
            cursor: 'pointer',
            opacity: submitting || !newComment.trim() ? 0.5 : 1,
          }}
        >
          {submitting ? 'Posting...' : 'Comment'}
        </button>
      </form>
    </section>
  );
}
