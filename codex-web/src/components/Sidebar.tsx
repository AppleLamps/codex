'use client';

import { Plus, MessageSquare, Archive, Settings } from 'lucide-react';
import clsx from 'clsx';
import type { Thread } from '@/types/codex';

interface SidebarProps {
  threads: Thread[];
  currentThreadId: string | null;
  onNewThread: () => void;
  onSelectThread: (threadId: string) => void;
  isConnected: boolean;
}

export function Sidebar({
  threads,
  currentThreadId,
  onNewThread,
  onSelectThread,
  isConnected,
}: SidebarProps) {
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
          <div className="w-8 h-8 bg-white rounded flex items-center justify-center">
            <span className="text-black font-bold text-sm">CX</span>
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
          disabled={!isConnected}
          className="w-full flex items-center justify-center gap-2 px-4 py-2 bg-white text-black rounded-lg font-medium hover:bg-gray-100 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
        >
          <Plus size={18} />
          New Session
        </button>
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
                <li key={thread.id}>
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
        <button className="w-full flex items-center gap-2 px-2 py-2 text-codex-muted hover:text-codex-text hover:bg-codex-hover rounded-lg transition-colors">
          <Settings size={16} />
          <span className="text-sm">Settings</span>
        </button>
      </div>
    </aside>
  );
}
