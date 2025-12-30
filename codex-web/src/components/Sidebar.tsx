'use client';

import { useState } from 'react';
import { Plus, MessageSquare, Archive, Settings, Search, GitPullRequest, Trash2, X } from 'lucide-react';
import clsx from 'clsx';
import type { Thread } from '@/types/codex';
import { AuthStatus } from './AuthStatus';

interface SidebarProps {
  threads: Thread[];
  currentThreadId: string | null;
  onNewThread: () => void;
  onSelectThread: (threadId: string) => void;
  onArchiveThread?: (threadId: string) => void;
  isConnected: boolean;
  isAuthenticated?: boolean;
  accountInfo?: { email?: string; name?: string; plan?: string } | null;
  onLogin?: () => void;
  onLogout?: () => void;
  onOpenSettings?: () => void;
  onOpenReview?: () => void;
  onOpenSearch?: () => void;
}

export function Sidebar({
  threads,
  currentThreadId,
  onNewThread,
  onSelectThread,
  onArchiveThread,
  isConnected,
  isAuthenticated = false,
  accountInfo = null,
  onLogin,
  onLogout,
  onOpenSettings,
  onOpenReview,
  onOpenSearch,
}: SidebarProps) {
  const [confirmArchive, setConfirmArchive] = useState<string | null>(null);

  const handleArchive = (threadId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    if (confirmArchive === threadId) {
      onArchiveThread?.(threadId);
      setConfirmArchive(null);
    } else {
      setConfirmArchive(threadId);
    }
  };

  const formatDate = (timestamp: number) => {
    const date = new Date(timestamp * 1000);
    const now = new Date();
    const diff = now.getTime() - date.getTime();
    const days = Math.floor(diff / (1000 * 60 * 60 * 24));

    if (days === 0) {
      return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    } else if (days === 1) {
      return 'Yesterday';
    } else if (days < 7) {
      return date.toLocaleDateString([], { weekday: 'short' });
    } else {
      return date.toLocaleDateString([], { month: 'short', day: 'numeric' });
    }
  };

  return (
    <aside className="w-64 h-full bg-codex-bg border-r border-codex-border flex flex-col">
      {/* Header */}
      <div className="p-4 border-b border-codex-border">
        <div className="flex items-center gap-2 mb-4">
          <div className="w-8 h-8 bg-codex-accent rounded flex items-center justify-center">
            <span className="text-codex-bg font-bold text-sm">CX</span>
          </div>
          <span className="font-semibold text-codex-text">CODEX</span>
          <div
            className={clsx(
              'ml-auto w-2 h-2 rounded-full',
              isConnected ? 'bg-codex-success' : 'bg-codex-error'
            )}
            title={isConnected ? 'Connected' : 'Disconnected'}
          />
        </div>
        <button
          onClick={onNewThread}
          disabled={!isConnected || !isAuthenticated}
          className="w-full flex items-center justify-center gap-2 px-4 py-2 bg-codex-accent text-codex-bg rounded-lg font-medium hover:opacity-80 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <Plus size={18} />
          New Session
        </button>
      </div>

      {/* Auth Status */}
      {onLogin && onLogout && (
        <div className="border-b border-codex-border">
          <AuthStatus
            isAuthenticated={isAuthenticated}
            accountInfo={accountInfo}
            onLogin={onLogin}
            onLogout={onLogout}
          />
        </div>
      )}

      {/* Quick Actions */}
      <div className="p-2 border-b border-codex-border space-y-1">
        {onOpenSearch && (
          <button
            onClick={onOpenSearch}
            disabled={!isConnected}
            className="w-full flex items-center gap-2 px-2 py-2 text-codex-muted hover:text-codex-text hover:bg-codex-hover rounded-lg transition-colors disabled:opacity-50"
          >
            <Search size={16} />
            <span className="text-sm">Search Files</span>
            <span className="ml-auto text-xs text-codex-muted">Ctrl+P</span>
          </button>
        )}
        {onOpenReview && (
          <button
            onClick={onOpenReview}
            disabled={!isConnected || !isAuthenticated}
            className="w-full flex items-center gap-2 px-2 py-2 text-codex-muted hover:text-codex-text hover:bg-codex-hover rounded-lg transition-colors disabled:opacity-50"
          >
            <GitPullRequest size={16} />
            <span className="text-sm">Code Review</span>
          </button>
        )}
      </div>

      {/* Sessions List */}
      <div className="flex-1 overflow-y-auto">
        <div className="p-2">
          <h3 className="px-2 py-1 text-xs font-medium text-codex-muted uppercase tracking-wider">
            Sessions
          </h3>
          {threads.length === 0 ? (
            <p className="px-2 py-4 text-sm text-codex-muted text-center">
              No sessions yet
            </p>
          ) : (
            <ul className="space-y-1">
              {threads.map((thread) => (
                <li key={thread.id} className="group relative">
                  <button
                    onClick={() => onSelectThread(thread.id)}
                    className={clsx(
                      'w-full flex items-start gap-2 px-2 py-2 rounded-lg text-left transition-colors',
                      currentThreadId === thread.id
                        ? 'bg-codex-hover text-codex-text'
                        : 'text-codex-muted hover:bg-codex-hover hover:text-codex-text'
                    )}
                  >
                    <MessageSquare size={16} className="mt-0.5 flex-shrink-0" />
                    <div className="flex-1 min-w-0">
                      <p className="text-sm truncate">
                        {thread.preview || 'New session'}
                      </p>
                      <p className="text-xs text-codex-muted">
                        {formatDate(thread.createdAt)}
                      </p>
                    </div>
                  </button>
                  {onArchiveThread && (
                    <div className="absolute right-1 top-1/2 -translate-y-1/2 opacity-0 group-hover:opacity-100 transition-opacity">
                      {confirmArchive === thread.id ? (
                        <div className="flex items-center gap-1 bg-codex-surface rounded px-1">
                          <button
                            onClick={(e) => handleArchive(thread.id, e)}
                            className="p-1 text-codex-error hover:bg-codex-error/20 rounded"
                            title="Confirm archive"
                          >
                            <Trash2 size={14} />
                          </button>
                          <button
                            onClick={(e) => { e.stopPropagation(); setConfirmArchive(null); }}
                            className="p-1 text-codex-muted hover:bg-codex-hover rounded"
                            title="Cancel"
                          >
                            <X size={14} />
                          </button>
                        </div>
                      ) : (
                        <button
                          onClick={(e) => handleArchive(thread.id, e)}
                          className="p-1 text-codex-muted hover:text-codex-error hover:bg-codex-hover rounded"
                          title="Archive thread"
                        >
                          <Archive size={14} />
                        </button>
                      )}
                    </div>
                  )}
                </li>
              ))}
            </ul>
          )}
        </div>
      </div>

      {/* Footer */}
      <div className="p-2 border-t border-codex-border">
        <button className="w-full flex items-center gap-2 px-2 py-2 text-codex-muted hover:text-codex-text hover:bg-codex-hover rounded-lg transition-colors">
          <Archive size={16} />
          <span className="text-sm">Archived</span>
        </button>
        <button
          onClick={onOpenSettings}
          className="w-full flex items-center gap-2 px-2 py-2 text-codex-muted hover:text-codex-text hover:bg-codex-hover rounded-lg transition-colors"
        >
          <Settings size={16} />
          <span className="text-sm">Settings</span>
        </button>
      </div>
    </aside>
  );
}
