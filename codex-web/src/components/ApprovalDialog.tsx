'use client';

import { AlertTriangle, Check, X, Terminal, FileEdit, Globe } from 'lucide-react';

interface ApprovalRequest {
  id: string;
  type: 'command' | 'fileWrite' | 'fileDelete' | 'network' | 'other';
  title: string;
  description?: string;
  details?: {
    command?: string;
    filePath?: string;
    url?: string;
  };
}

interface ApprovalDialogProps {
  request: ApprovalRequest | null;
  onApprove: (id: string) => void;
  onDeny: (id: string) => void;
}

export function ApprovalDialog({ request, onApprove, onDeny }: ApprovalDialogProps) {
  if (!request) return null;

  const getIcon = () => {
    switch (request.type) {
      case 'command':
        return <Terminal size={24} className="text-codex-warning" />;
      case 'fileWrite':
      case 'fileDelete':
        return <FileEdit size={24} className="text-codex-warning" />;
      case 'network':
        return <Globe size={24} className="text-codex-warning" />;
      default:
        return <AlertTriangle size={24} className="text-codex-warning" />;
    }
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-codex-surface border border-codex-border rounded-lg w-full max-w-md mx-4 overflow-hidden">
        {/* Header */}
        <div className="flex items-center gap-3 p-4 border-b border-codex-border bg-codex-warning/5">
          {getIcon()}
          <div>
            <h2 className="text-lg font-semibold text-codex-text">Approval Required</h2>
            <p className="text-sm text-codex-muted">{request.title}</p>
          </div>
        </div>

        {/* Content */}
        <div className="p-4">
          {request.description && (
            <p className="text-sm text-codex-text mb-4">{request.description}</p>
          )}

          {request.details?.command && (
            <div className="mb-4">
              <p className="text-xs font-medium text-codex-muted mb-1">Command</p>
              <pre className="p-3 bg-codex-bg rounded-lg text-sm text-codex-text overflow-x-auto font-mono">
                {request.details.command}
              </pre>
            </div>
          )}

          {request.details?.filePath && (
            <div className="mb-4">
              <p className="text-xs font-medium text-codex-muted mb-1">File Path</p>
              <p className="p-3 bg-codex-bg rounded-lg text-sm text-codex-text font-mono break-all">
                {request.details.filePath}
              </p>
            </div>
          )}

          {request.details?.url && (
            <div className="mb-4">
              <p className="text-xs font-medium text-codex-muted mb-1">URL</p>
              <p className="p-3 bg-codex-bg rounded-lg text-sm text-codex-text font-mono break-all">
                {request.details.url}
              </p>
            </div>
          )}
        </div>

        {/* Actions */}
        <div className="flex gap-2 p-4 border-t border-codex-border">
          <button
            onClick={() => onDeny(request.id)}
            className="flex-1 flex items-center justify-center gap-2 px-4 py-2 border border-codex-error text-codex-error rounded-lg hover:bg-codex-error/10 transition-colors"
          >
            <X size={16} />
            Deny
          </button>
          <button
            onClick={() => onApprove(request.id)}
            className="flex-1 flex items-center justify-center gap-2 px-4 py-2 bg-codex-success text-white rounded-lg hover:bg-codex-success/80 transition-colors"
          >
            <Check size={16} />
            Approve
          </button>
        </div>
      </div>
    </div>
  );
}
