'use client';

import { AlertTriangle, Check, X, Terminal, FileEdit, Globe } from 'lucide-react';

interface ApprovalRequest {
  itemId: string;
  type: 'commandExecution' | 'fileChange';
  description: string;
}

interface ApprovalDialogProps {
  request: ApprovalRequest | null;
  onApprove: () => void;
  onDeny: () => void;
}

export function ApprovalDialog({ request, onApprove, onDeny }: ApprovalDialogProps) {
  if (!request) return null;

  const getIcon = () => {
    switch (request.type) {
      case 'commandExecution':
        return <Terminal size={24} className="text-codex-warning" />;
      case 'fileChange':
        return <FileEdit size={24} className="text-codex-warning" />;
      default:
        return <AlertTriangle size={24} className="text-codex-warning" />;
    }
  };

  const getTitle = () => {
    switch (request.type) {
      case 'commandExecution':
        return 'Command Execution';
      case 'fileChange':
        return 'File Changes';
      default:
        return 'Action Required';
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
            <p className="text-sm text-codex-muted">{getTitle()}</p>
          </div>
        </div>

        {/* Content */}
        <div className="p-4">
          <div className="mb-4">
            <p className="text-xs font-medium text-codex-muted mb-1">
              {request.type === 'commandExecution' ? 'Command' : 'Files'}
            </p>
            <pre className="p-3 bg-codex-bg rounded-lg text-sm text-codex-text overflow-x-auto font-mono whitespace-pre-wrap break-all">
              {request.description}
            </pre>
          </div>
          <p className="text-sm text-codex-muted">
            {request.type === 'commandExecution'
              ? 'Do you want to allow this command to be executed?'
              : 'Do you want to allow these file changes?'}
          </p>
        </div>

        {/* Actions */}
        <div className="flex gap-2 p-4 border-t border-codex-border">
          <button
            onClick={onDeny}
            className="flex-1 flex items-center justify-center gap-2 px-4 py-2 border border-codex-error text-codex-error rounded-lg hover:bg-codex-error/10 transition-colors"
          >
            <X size={16} />
            Deny
          </button>
          <button
            onClick={onApprove}
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
