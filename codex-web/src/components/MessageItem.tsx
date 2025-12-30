'use client';

import {
  User,
  Bot,
  Terminal,
  FileText,
  Search,
  AlertCircle,
  CheckCircle,
  Loader2,
  ChevronDown,
  ChevronRight,
  Brain,
} from 'lucide-react';
import { useState } from 'react';
import clsx from 'clsx';
import type { ThreadItem } from '@/types/codex';

interface MessageItemProps {
  item: ThreadItem;
}

export function MessageItem({ item }: MessageItemProps) {
  const [isExpanded, setIsExpanded] = useState(true);

  switch (item.type) {
    case 'userMessage':
      return (
        <div className="flex gap-3 px-4 py-3">
          <div className="w-8 h-8 bg-codex-border rounded-full flex items-center justify-center flex-shrink-0">
            <User size={16} className="text-codex-text" />
          </div>
          <div className="flex-1 min-w-0">
            <p className="text-sm font-medium text-codex-text mb-1">You</p>
            {item.content.map((c, i) => (
              <div key={i}>
                {c.type === 'text' && (
                  <p className="text-codex-text whitespace-pre-wrap">{c.text}</p>
                )}
                {c.type === 'localImage' && (
                  <p className="text-codex-muted text-sm">[Image: {c.path}]</p>
                )}
              </div>
            ))}
          </div>
        </div>
      );

    case 'agentMessage':
      return (
        <div className="flex gap-3 px-4 py-3 bg-codex-surface">
          <div className="w-8 h-8 bg-codex-accent rounded-full flex items-center justify-center flex-shrink-0">
            <Bot size={16} className="text-codex-bg" />
          </div>
          <div className="flex-1 min-w-0">
            <p className="text-sm font-medium text-codex-text mb-1">Codex</p>
            <div className="text-codex-text whitespace-pre-wrap prose prose-invert prose-sm max-w-none">
              {item.text || (
                <span className="inline-flex items-center gap-1 text-codex-muted">
                  <Loader2 size={14} className="animate-spin" />
                  Thinking...
                </span>
              )}
            </div>
          </div>
        </div>
      );

    case 'reasoning':
      return (
        <div className="px-4 py-2">
          <button
            onClick={() => setIsExpanded(!isExpanded)}
            className="flex items-center gap-2 text-codex-muted hover:text-codex-text transition-colors"
          >
            {isExpanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
            <Brain size={14} />
            <span className="text-xs font-medium">Reasoning</span>
          </button>
          {isExpanded && (item.summary || item.content) && (
            <div className="mt-2 ml-6 p-3 bg-codex-surface rounded border border-codex-border">
              <p className="text-sm text-codex-muted whitespace-pre-wrap">
                {item.summary || item.content}
              </p>
            </div>
          )}
        </div>
      );

    case 'commandExecution':
      return (
        <div className="px-4 py-2">
          <button
            onClick={() => setIsExpanded(!isExpanded)}
            className="flex items-center gap-2 text-codex-muted hover:text-codex-text transition-colors w-full"
          >
            {isExpanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
            <Terminal size={14} />
            <code className="text-xs bg-codex-surface px-2 py-0.5 rounded flex-1 text-left truncate">
              {item.command}
            </code>
            <StatusBadge status={item.status} exitCode={item.exitCode} />
          </button>
          {isExpanded && item.aggregatedOutput && (
            <div className="mt-2 ml-6">
              <pre className="p-3 bg-black rounded border border-codex-border text-xs text-codex-text overflow-x-auto font-mono">
                {item.aggregatedOutput}
              </pre>
              {item.durationMs && (
                <p className="mt-1 text-xs text-codex-muted">
                  Completed in {(item.durationMs / 1000).toFixed(2)}s
                </p>
              )}
            </div>
          )}
        </div>
      );

    case 'fileChange':
      return (
        <div className="px-4 py-2">
          <button
            onClick={() => setIsExpanded(!isExpanded)}
            className="flex items-center gap-2 text-codex-muted hover:text-codex-text transition-colors w-full"
          >
            {isExpanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />}
            <FileText size={14} />
            <span className="text-xs font-medium flex-1 text-left">
              {item.changes.length} file{item.changes.length !== 1 ? 's' : ''} changed
            </span>
            <StatusBadge status={item.status} />
          </button>
          {isExpanded && (
            <div className="mt-2 ml-6 space-y-2">
              {item.changes.map((change, i) => (
                <div key={i} className="p-3 bg-codex-surface rounded border border-codex-border">
                  <div className="flex items-center gap-2 mb-2">
                    <span
                      className={clsx(
                        'text-xs px-1.5 py-0.5 rounded font-medium',
                        change.kind === 'add' && 'bg-green-900/50 text-green-400',
                        change.kind === 'delete' && 'bg-red-900/50 text-red-400',
                        change.kind === 'update' && 'bg-yellow-900/50 text-yellow-400'
                      )}
                    >
                      {change.kind.toUpperCase()}
                    </span>
                    <code className="text-xs text-codex-text truncate">{change.path}</code>
                  </div>
                  {change.diff && (
                    <pre className="text-xs text-codex-muted overflow-x-auto font-mono">
                      {change.diff}
                    </pre>
                  )}
                </div>
              ))}
            </div>
          )}
        </div>
      );

    case 'webSearch':
      return (
        <div className="px-4 py-2 flex items-center gap-2">
          <Search size={14} className="text-codex-muted" />
          <span className="text-xs text-codex-muted">Searching: </span>
          <span className="text-xs text-codex-text">{item.query}</span>
        </div>
      );

    case 'mcpToolCall':
      return (
        <div className="px-4 py-2">
          <div className="flex items-center gap-2">
            <Terminal size={14} className="text-codex-muted" />
            <span className="text-xs text-codex-muted">
              {item.server}/{item.tool}
            </span>
            <StatusBadge status={item.status} />
          </div>
        </div>
      );

    case 'todoList':
      return (
        <div className="px-4 py-2">
          <div className="flex items-center gap-2 mb-2">
            <CheckCircle size={14} className="text-codex-muted" />
            <span className="text-xs font-medium text-codex-muted">Plan</span>
          </div>
          <ul className="ml-6 space-y-1">
            {item.items.map((todo, i) => (
              <li key={i} className="flex items-center gap-2 text-sm">
                <span
                  className={clsx(
                    'w-4 h-4 rounded border flex items-center justify-center',
                    todo.completed
                      ? 'bg-codex-success border-codex-success'
                      : 'border-codex-border'
                  )}
                >
                  {todo.completed && <CheckCircle size={12} className="text-white" />}
                </span>
                <span className={clsx(todo.completed && 'line-through text-codex-muted')}>
                  {todo.text}
                </span>
              </li>
            ))}
          </ul>
        </div>
      );

    case 'error':
      return (
        <div className="px-4 py-2 flex items-center gap-2 text-codex-error">
          <AlertCircle size={14} />
          <span className="text-sm">{item.message}</span>
        </div>
      );

    default:
      return null;
  }
}

function StatusBadge({
  status,
  exitCode,
}: {
  status: string;
  exitCode?: number;
}) {
  const getStatusConfig = () => {
    switch (status) {
      case 'inProgress':
        return {
          icon: <Loader2 size={12} className="animate-spin" />,
          text: 'Running',
          className: 'text-yellow-400',
        };
      case 'completed':
        return {
          icon: <CheckCircle size={12} />,
          text: exitCode !== undefined ? `Exit ${exitCode}` : 'Done',
          className: exitCode === 0 ? 'text-codex-success' : 'text-codex-warning',
        };
      case 'failed':
        return {
          icon: <AlertCircle size={12} />,
          text: 'Failed',
          className: 'text-codex-error',
        };
      case 'declined':
        return {
          icon: <AlertCircle size={12} />,
          text: 'Declined',
          className: 'text-codex-muted',
        };
      default:
        return null;
    }
  };

  const config = getStatusConfig();
  if (!config) return null;

  return (
    <span className={clsx('flex items-center gap-1 text-xs', config.className)}>
      {config.icon}
      {config.text}
    </span>
  );
}
