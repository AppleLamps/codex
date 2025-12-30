'use client';

import { useState } from 'react';
import { X, GitBranch, GitCommit, FileCode, MessageSquare, Loader2 } from 'lucide-react';

interface ReviewDialogProps {
  isOpen: boolean;
  onClose: () => void;
  onStartReview: (params: {
    target: 'uncommitted' | 'base' | 'commit' | 'custom';
    baseBranch?: string;
    commitSha?: string;
    instructions?: string;
  }) => Promise<void>;
}

export function ReviewDialog({ isOpen, onClose, onStartReview }: ReviewDialogProps) {
  const [target, setTarget] = useState<'uncommitted' | 'base' | 'commit' | 'custom'>('uncommitted');
  const [baseBranch, setBaseBranch] = useState('main');
  const [commitSha, setCommitSha] = useState('');
  const [instructions, setInstructions] = useState('');
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  if (!isOpen) return null;

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setLoading(true);
    setError(null);

    try {
      await onStartReview({
        target,
        baseBranch: target === 'base' ? baseBranch : undefined,
        commitSha: target === 'commit' ? commitSha : undefined,
        instructions: instructions.trim() || undefined,
      });
      onClose();
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Failed to start review');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-codex-surface border border-codex-border rounded-lg w-full max-w-md mx-4 overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-codex-border">
          <h2 className="text-lg font-semibold text-codex-text">Start Code Review</h2>
          <button
            onClick={onClose}
            className="text-codex-muted hover:text-codex-text transition-colors"
          >
            <X size={20} />
          </button>
        </div>

        {/* Content */}
        <form onSubmit={handleSubmit} className="p-4 space-y-4">
          {error && (
            <div className="p-3 bg-codex-error/10 border border-codex-error/30 rounded-lg text-codex-error text-sm">
              {error}
            </div>
          )}

          <div>
            <label className="block text-sm font-medium text-codex-text mb-2">
              Review Target
            </label>
            <div className="space-y-2">
              <label className="flex items-center gap-2 p-3 bg-codex-hover rounded-lg cursor-pointer hover:bg-codex-border transition-colors">
                <input
                  type="radio"
                  name="target"
                  value="uncommitted"
                  checked={target === 'uncommitted'}
                  onChange={() => setTarget('uncommitted')}
                  className="text-codex-accent"
                />
                <FileCode size={18} className="text-codex-muted" />
                <div>
                  <p className="text-sm text-codex-text">Uncommitted Changes</p>
                  <p className="text-xs text-codex-muted">Review staged and unstaged changes</p>
                </div>
              </label>

              <label className="flex items-center gap-2 p-3 bg-codex-hover rounded-lg cursor-pointer hover:bg-codex-border transition-colors">
                <input
                  type="radio"
                  name="target"
                  value="base"
                  checked={target === 'base'}
                  onChange={() => setTarget('base')}
                  className="text-codex-accent"
                />
                <GitBranch size={18} className="text-codex-muted" />
                <div>
                  <p className="text-sm text-codex-text">Compare to Branch</p>
                  <p className="text-xs text-codex-muted">Review changes since base branch</p>
                </div>
              </label>

              <label className="flex items-center gap-2 p-3 bg-codex-hover rounded-lg cursor-pointer hover:bg-codex-border transition-colors">
                <input
                  type="radio"
                  name="target"
                  value="commit"
                  checked={target === 'commit'}
                  onChange={() => setTarget('commit')}
                  className="text-codex-accent"
                />
                <GitCommit size={18} className="text-codex-muted" />
                <div>
                  <p className="text-sm text-codex-text">Specific Commit</p>
                  <p className="text-xs text-codex-muted">Review a specific commit</p>
                </div>
              </label>

              <label className="flex items-center gap-2 p-3 bg-codex-hover rounded-lg cursor-pointer hover:bg-codex-border transition-colors">
                <input
                  type="radio"
                  name="target"
                  value="custom"
                  checked={target === 'custom'}
                  onChange={() => setTarget('custom')}
                  className="text-codex-accent"
                />
                <MessageSquare size={18} className="text-codex-muted" />
                <div>
                  <p className="text-sm text-codex-text">Custom Instructions</p>
                  <p className="text-xs text-codex-muted">Provide specific review instructions</p>
                </div>
              </label>
            </div>
          </div>

          {target === 'base' && (
            <div>
              <label className="block text-sm font-medium text-codex-text mb-1">
                Base Branch
              </label>
              <input
                type="text"
                value={baseBranch}
                onChange={(e) => setBaseBranch(e.target.value)}
                placeholder="main"
                className="w-full px-3 py-2 bg-codex-bg border border-codex-border rounded-lg text-codex-text placeholder-codex-muted focus:outline-none focus:ring-2 focus:ring-codex-accent"
              />
            </div>
          )}

          {target === 'commit' && (
            <div>
              <label className="block text-sm font-medium text-codex-text mb-1">
                Commit SHA
              </label>
              <input
                type="text"
                value={commitSha}
                onChange={(e) => setCommitSha(e.target.value)}
                placeholder="abc1234..."
                className="w-full px-3 py-2 bg-codex-bg border border-codex-border rounded-lg text-codex-text placeholder-codex-muted focus:outline-none focus:ring-2 focus:ring-codex-accent font-mono"
              />
            </div>
          )}

          <div>
            <label className="block text-sm font-medium text-codex-text mb-1">
              Additional Instructions (optional)
            </label>
            <textarea
              value={instructions}
              onChange={(e) => setInstructions(e.target.value)}
              placeholder="Focus on security issues, performance optimizations..."
              rows={3}
              className="w-full px-3 py-2 bg-codex-bg border border-codex-border rounded-lg text-codex-text placeholder-codex-muted focus:outline-none focus:ring-2 focus:ring-codex-accent resize-none"
            />
          </div>

          <div className="flex gap-2">
            <button
              type="button"
              onClick={onClose}
              className="flex-1 px-4 py-2 border border-codex-border text-codex-text rounded-lg hover:bg-codex-hover transition-colors"
            >
              Cancel
            </button>
            <button
              type="submit"
              disabled={loading}
              className="flex-1 px-4 py-2 bg-codex-accent text-codex-bg rounded-lg font-medium hover:opacity-80 transition-colors disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
            >
              {loading && <Loader2 size={16} className="animate-spin" />}
              Start Review
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
