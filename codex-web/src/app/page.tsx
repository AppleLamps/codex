'use client';

import { useEffect, useState, useCallback } from 'react';
import { Sidebar } from '@/components/Sidebar';
import { ChatThread } from '@/components/ChatThread';
import { InputBox } from '@/components/InputBox';
import { LoginDialog } from '@/components/LoginDialog';
import { SettingsPanel } from '@/components/SettingsPanel';
import { FileSearchDialog } from '@/components/FileSearchDialog';
import { ReviewDialog } from '@/components/ReviewDialog';
import { ModelSelector } from '@/components/ModelSelector';
import { ApprovalDialog } from '@/components/ApprovalDialog';
import { useCodex } from '@/lib/hooks/useCodex';
import { useTheme } from '@/lib/theme-context';
import { AlertCircle, RefreshCw, Sun, Moon, Coins } from 'lucide-react';

interface Model {
  id: string;
  name: string;
  description?: string;
}

export default function Home() {
  const {
    sessionId,
    thread,
    turn,
    items,
    threads,
    isConnected,
    isLoading,
    error,
    pendingApproval,
    tokenUsage,
    plan,
    connect,
    disconnect,
    startThread,
    resumeThread,
    archiveThread,
    sendMessage,
    interruptTurn,
    loadThreads,
    respondToApproval,
  } = useCodex();

  // UI state
  const [showLogin, setShowLogin] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [showSearch, setShowSearch] = useState(false);
  const [showReview, setShowReview] = useState(false);

  // Auth state
  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [accountInfo, setAccountInfo] = useState<{ email?: string; name?: string; plan?: string } | null>(null);

  // Models state
  const [models, setModels] = useState<Model[]>([]);
  const [selectedModel, setSelectedModel] = useState<string | null>(null);
  const [loadingModels, setLoadingModels] = useState(false);

  // Theme
  const { theme, toggleTheme } = useTheme();

  // Auto-connect on mount
  useEffect(() => {
    connect();
    return () => disconnect();
  }, []);

  // Check auth status when connected
  useEffect(() => {
    if (isConnected && sessionId) {
      checkAuthStatus();
      loadModels();
    }
  }, [isConnected, sessionId]);

  // Load threads when connected and authenticated
  useEffect(() => {
    if (isConnected && isAuthenticated) {
      loadThreads();
    }
  }, [isConnected, isAuthenticated, loadThreads]);

  // Keyboard shortcuts
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 'p') {
        e.preventDefault();
        setShowSearch(true);
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, []);

  const checkAuthStatus = async () => {
    if (!sessionId) return;

    try {
      const response = await fetch(`/api/codex/auth?sessionId=${sessionId}`);
      const data = await response.json();

      if (data.status === 'authenticated' || data.account) {
        setIsAuthenticated(true);
        setAccountInfo(data.account || null);
      } else {
        setIsAuthenticated(false);
        setAccountInfo(null);
      }
    } catch (err) {
      console.error('Failed to check auth status:', err);
      // Assume authenticated if we can't check (backend might not support this endpoint)
      setIsAuthenticated(true);
    }
  };

  const loadModels = async () => {
    if (!sessionId) return;
    setLoadingModels(true);

    try {
      const response = await fetch(`/api/codex/models?sessionId=${sessionId}`);
      const data = await response.json();

      if (data.models) {
        setModels(data.models);
        if (data.defaultModel) {
          setSelectedModel(data.defaultModel);
        }
      }
    } catch (err) {
      console.error('Failed to load models:', err);
    } finally {
      setLoadingModels(false);
    }
  };

  const handleLoginApiKey = async (apiKey: string) => {
    if (!sessionId) return { success: false, error: 'Not connected' };

    try {
      const response = await fetch('/api/codex/auth', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ sessionId, method: 'apiKey', apiKey }),
      });
      const data = await response.json();

      if (data.success) {
        setIsAuthenticated(true);
        await checkAuthStatus();
      }

      return data;
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : 'Login failed' };
    }
  };

  const handleLoginDevice = async () => {
    if (!sessionId) throw new Error('Not connected');

    const response = await fetch('/api/codex/auth', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ sessionId, method: 'device' }),
    });
    const data = await response.json();

    if (data.error) {
      throw new Error(data.error);
    }

    return data;
  };

  const handleLogout = async () => {
    if (!sessionId) return;

    try {
      await fetch(`/api/codex/auth?sessionId=${sessionId}`, { method: 'DELETE' });
      setIsAuthenticated(false);
      setAccountInfo(null);
      // Clear session on logout - pass true to clear localStorage
      disconnect(true);
    } catch (err) {
      console.error('Failed to logout:', err);
    }
  };

  const handleSelectModel = async (modelId: string) => {
    if (!sessionId) return;
    setSelectedModel(modelId);

    try {
      await fetch('/api/codex/models', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ sessionId, model: modelId }),
      });
    } catch (err) {
      console.error('Failed to set model:', err);
    }
  };

  const handleStartReview = async (params: {
    target: 'uncommitted' | 'base' | 'commit' | 'custom';
    baseBranch?: string;
    commitSha?: string;
    instructions?: string;
  }) => {
    if (!sessionId) throw new Error('Not connected');

    const response = await fetch('/api/codex/review', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ sessionId, ...params }),
    });

    if (!response.ok) {
      const data = await response.json();
      throw new Error(data.error || 'Failed to start review');
    }
  };

  const handleNewThread = async () => {
    await startThread({ model: selectedModel || undefined });
  };

  const handleSelectThread = async (threadId: string) => {
    await resumeThread(threadId);
    await loadThreads();
  };

  const handleSendMessage = async (message: string) => {
    if (!thread) {
      // Auto-start thread if none exists
      await startThread({ model: selectedModel || undefined });
      // Wait a bit for thread to be created
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
    await sendMessage(message);
  };

  const isProcessing = turn?.status === 'inProgress';

  return (
    <main className="h-screen flex">
      {/* Sidebar */}
      <Sidebar
        threads={threads}
        currentThreadId={thread?.id || null}
        onNewThread={handleNewThread}
        onSelectThread={handleSelectThread}
        onArchiveThread={archiveThread}
        isConnected={isConnected}
        isAuthenticated={isAuthenticated}
        accountInfo={accountInfo}
        onLogin={() => setShowLogin(true)}
        onLogout={handleLogout}
        onOpenSettings={() => setShowSettings(true)}
        onOpenSearch={() => setShowSearch(true)}
        onOpenReview={() => setShowReview(true)}
      />

      {/* Main content */}
      <div className="flex-1 flex flex-col min-w-0">
        {/* Header */}
        <header className="h-14 border-b border-codex-border flex items-center justify-between px-4">
          <div className="flex items-center gap-3">
            <h1 className="font-semibold text-codex-text">
              {thread ? thread.preview || 'New Session' : 'Codex'}
            </h1>
            {sessionId && (
              <span className="text-xs text-codex-muted font-mono">
                {thread?.id ? `#${thread.id.slice(0, 8)}` : ''}
              </span>
            )}
          </div>
          <div className="flex items-center gap-2">
            {/* Token Usage */}
            {tokenUsage && (
              <div className="flex items-center gap-1.5 px-2 py-1 bg-codex-surface border border-codex-border rounded-lg text-xs text-codex-muted">
                <Coins size={12} />
                <span>{tokenUsage.totalTokens.toLocaleString()} tokens</span>
              </div>
            )}
            {/* Theme Toggle */}
            <button
              onClick={toggleTheme}
              className="p-2 rounded-lg hover:bg-codex-hover transition-colors"
              aria-label={theme === 'light' ? 'Switch to dark mode' : 'Switch to light mode'}
            >
              {theme === 'light' ? <Moon size={18} /> : <Sun size={18} />}
            </button>
            {/* Model Selector */}
            {isConnected && models.length > 0 && (
              <ModelSelector
                models={models}
                selectedModel={selectedModel}
                onSelectModel={handleSelectModel}
                loading={loadingModels}
                disabled={!isAuthenticated}
              />
            )}
            {!isConnected && !isLoading && (
              <button
                onClick={connect}
                className="flex items-center gap-2 px-3 py-1.5 text-sm bg-codex-surface border border-codex-border rounded-lg hover:bg-codex-hover transition-colors"
              >
                <RefreshCw size={14} />
                Reconnect
              </button>
            )}
          </div>
        </header>

        {/* Error banner */}
        {error && (
          <div className="bg-red-900/20 border-b border-red-900/50 px-4 py-2 flex items-center gap-2 text-codex-error">
            <AlertCircle size={16} />
            <span className="text-sm">{error}</span>
            <button
              onClick={() => window.location.reload()}
              className="ml-auto text-xs underline hover:no-underline"
            >
              Reload
            </button>
          </div>
        )}

        {/* Auth required banner */}
        {isConnected && !isAuthenticated && (
          <div className="bg-codex-warning/10 border-b border-codex-warning/30 px-4 py-2 flex items-center gap-2 text-codex-warning">
            <AlertCircle size={16} />
            <span className="text-sm">Authentication required to use Codex</span>
            <button
              onClick={() => setShowLogin(true)}
              className="ml-auto px-3 py-1 text-xs bg-codex-warning text-white rounded hover:bg-codex-warning/80 transition-colors"
            >
              Login
            </button>
          </div>
        )}

        {/* Chat thread */}
        <ChatThread items={items} turn={turn} isLoading={isLoading} plan={plan} />

        {/* Input box */}
        <InputBox
          onSend={handleSendMessage}
          onInterrupt={interruptTurn}
          disabled={!isConnected || isLoading || !isAuthenticated}
          isProcessing={isProcessing}
          placeholder={
            !isConnected
              ? 'Connecting to Codex...'
              : !isAuthenticated
              ? 'Please login to continue...'
              : !thread
              ? 'Start a new session to begin...'
              : 'Ask Codex to write code...'
          }
        />
      </div>

      {/* Dialogs */}
      <LoginDialog
        isOpen={showLogin}
        onClose={() => setShowLogin(false)}
        onLoginApiKey={handleLoginApiKey}
        onLoginDevice={handleLoginDevice}
      />

      <SettingsPanel
        isOpen={showSettings}
        onClose={() => setShowSettings(false)}
        sessionId={sessionId}
      />

      <FileSearchDialog
        isOpen={showSearch}
        onClose={() => setShowSearch(false)}
        sessionId={sessionId}
        onSelectFile={(path) => {
          // Could insert file path into input or open file
          console.log('Selected file:', path);
        }}
      />

      <ReviewDialog
        isOpen={showReview}
        onClose={() => setShowReview(false)}
        onStartReview={handleStartReview}
      />

      <ApprovalDialog
        request={pendingApproval}
        onApprove={() => respondToApproval(true)}
        onDeny={() => respondToApproval(false)}
      />
    </main>
  );
}
