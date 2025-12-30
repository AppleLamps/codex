'use client';

import { useEffect } from 'react';
import { Sidebar } from '@/components/Sidebar';
import { ChatThread } from '@/components/ChatThread';
import { InputBox } from '@/components/InputBox';
import { useCodex } from '@/lib/hooks/useCodex';
import { AlertCircle, RefreshCw } from 'lucide-react';

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
    connect,
    disconnect,
    startThread,
    resumeThread,
    sendMessage,
    interruptTurn,
    loadThreads,
  } = useCodex();

  // Auto-connect on mount
  useEffect(() => {
    connect();
    return () => disconnect();
  }, []);

  // Load threads when connected
  useEffect(() => {
    if (isConnected) {
      loadThreads();
    }
  }, [isConnected, loadThreads]);

  const handleNewThread = async () => {
    await startThread();
  };

  const handleSelectThread = async (threadId: string) => {
    await resumeThread(threadId);
    await loadThreads();
  };

  const handleSendMessage = async (message: string) => {
    if (!thread) {
      // Auto-start thread if none exists
      await startThread();
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
        isConnected={isConnected}
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

        {/* Chat thread */}
        <ChatThread items={items} turn={turn} isLoading={isLoading} />

        {/* Input box */}
        <InputBox
          onSend={handleSendMessage}
          onInterrupt={interruptTurn}
          disabled={!isConnected || isLoading}
          isProcessing={isProcessing}
          placeholder={
            !isConnected
              ? 'Connecting to Codex...'
              : !thread
              ? 'Start a new session to begin...'
              : 'Ask Codex to write code...'
          }
        />
      </div>
    </main>
  );
}
