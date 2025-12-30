'use client';

import { User, LogOut, AlertCircle } from 'lucide-react';

interface AuthStatusProps {
  isAuthenticated: boolean;
  accountInfo?: {
    email?: string;
    name?: string;
    plan?: string;
  } | null;
  onLogin: () => void;
  onLogout: () => void;
}

export function AuthStatus({
  isAuthenticated,
  accountInfo,
  onLogin,
  onLogout,
}: AuthStatusProps) {
  if (!isAuthenticated) {
    return (
      <div className="flex items-center gap-2 p-2">
        <button
          onClick={onLogin}
          className="flex items-center gap-2 px-3 py-1.5 bg-codex-accent text-codex-bg rounded-lg text-sm font-medium hover:opacity-80 transition-colors"
        >
          <AlertCircle size={14} />
          Login Required
        </button>
      </div>
    );
  }

  return (
    <div className="flex items-center gap-2 p-2">
      <div className="flex items-center gap-2 flex-1 min-w-0">
        <div className="w-8 h-8 bg-codex-accent/20 rounded-full flex items-center justify-center flex-shrink-0">
          <User size={16} className="text-codex-accent" />
        </div>
        <div className="flex-1 min-w-0">
          <p className="text-sm font-medium text-codex-text truncate">
            {accountInfo?.name || accountInfo?.email || 'Logged In'}
          </p>
          {accountInfo?.plan && (
            <p className="text-xs text-codex-muted truncate">{accountInfo.plan}</p>
          )}
        </div>
      </div>
      <button
        onClick={onLogout}
        className="p-1.5 text-codex-muted hover:text-codex-text hover:bg-codex-hover rounded transition-colors"
        title="Logout"
      >
        <LogOut size={16} />
      </button>
    </div>
  );
}
