'use client';

import { useState, useEffect, useCallback, useRef } from 'react';
import type { Thread, Turn, ThreadItem, CodexEvent } from '@/types/codex';

interface UseCodexState {
  sessionId: string | null;
  thread: Thread | null;
  turn: Turn | null;
  items: ThreadItem[];
  threads: Thread[];
  isConnected: boolean;
  isLoading: boolean;
  error: string | null;
}

interface UseCodexReturn extends UseCodexState {
  connect: () => Promise<void>;
  disconnect: () => void;
  startThread: (cwd?: string) => Promise<void>;
  resumeThread: (threadId: string) => Promise<void>;
  sendMessage: (message: string) => Promise<void>;
  interruptTurn: () => Promise<void>;
  loadThreads: () => Promise<void>;
}

export function useCodex(): UseCodexReturn {
  const [state, setState] = useState<UseCodexState>({
    sessionId: null,
    thread: null,
    turn: null,
    items: [],
    threads: [],
    isConnected: false,
    isLoading: false,
    error: null,
  });

  const eventSourceRef = useRef<EventSource | null>(null);
  const itemsMapRef = useRef<Map<string, ThreadItem>>(new Map());

  // Connect to session and start SSE
  const connect = useCallback(async () => {
    try {
      setState(s => ({ ...s, isLoading: true, error: null }));

      // Create session
      const response = await fetch('/api/codex/session', { method: 'POST' });
      const { sessionId, error } = await response.json();

      if (error) throw new Error(error);

      // Connect to SSE
      const es = new EventSource(`/api/codex/events?sessionId=${sessionId}`);

      es.onopen = () => {
        setState(s => ({ ...s, sessionId, isConnected: true, isLoading: false }));
      };

      es.onerror = () => {
        setState(s => ({ ...s, error: 'Connection lost', isConnected: false }));
      };

      es.onmessage = (event) => {
        try {
          const data: CodexEvent & { type: string } = JSON.parse(event.data);
          handleEvent(data);
        } catch (err) {
          console.error('[useCodex] Failed to parse event:', err);
        }
      };

      eventSourceRef.current = es;
    } catch (err) {
      setState(s => ({
        ...s,
        isLoading: false,
        error: err instanceof Error ? err.message : 'Failed to connect',
      }));
    }
  }, []);

  // Handle incoming events
  const handleEvent = useCallback((event: CodexEvent & { type: string }) => {
    switch (event.type) {
      case 'connected':
        // Initial connection confirmation
        break;

      case 'thread/started':
        setState(s => ({ ...s, thread: (event as { thread: Thread }).thread }));
        itemsMapRef.current.clear();
        break;

      case 'turn/started':
        setState(s => ({ ...s, turn: (event as { turn: Turn }).turn }));
        break;

      case 'turn/completed':
        setState(s => ({ ...s, turn: (event as { turn: Turn }).turn }));
        break;

      case 'item/started':
      case 'item/completed': {
        const item = (event as { item: ThreadItem }).item;
        itemsMapRef.current.set(item.id, item);
        setState(s => ({ ...s, items: Array.from(itemsMapRef.current.values()) }));
        break;
      }

      case 'item/agentMessage/delta': {
        const { itemId, delta } = event as { itemId: string; delta: string };
        const existing = itemsMapRef.current.get(itemId);
        if (existing && existing.type === 'agentMessage') {
          existing.text = (existing.text || '') + delta;
          itemsMapRef.current.set(itemId, { ...existing });
          setState(s => ({ ...s, items: Array.from(itemsMapRef.current.values()) }));
        }
        break;
      }

      case 'item/commandExecution/outputDelta': {
        const { itemId, delta } = event as { itemId: string; delta: string };
        const existing = itemsMapRef.current.get(itemId);
        if (existing && existing.type === 'commandExecution') {
          existing.aggregatedOutput = (existing.aggregatedOutput || '') + delta;
          itemsMapRef.current.set(itemId, { ...existing });
          setState(s => ({ ...s, items: Array.from(itemsMapRef.current.values()) }));
        }
        break;
      }

      case 'error':
        setState(s => ({ ...s, error: (event as { error: { message: string } }).error.message }));
        break;

      case 'exit':
        setState(s => ({ ...s, isConnected: false }));
        break;
    }
  }, []);

  // Disconnect and cleanup
  const disconnect = useCallback(() => {
    if (eventSourceRef.current) {
      eventSourceRef.current.close();
      eventSourceRef.current = null;
    }

    if (state.sessionId) {
      fetch(`/api/codex/session?sessionId=${state.sessionId}`, { method: 'DELETE' });
    }

    setState({
      sessionId: null,
      thread: null,
      turn: null,
      items: [],
      threads: [],
      isConnected: false,
      isLoading: false,
      error: null,
    });
    itemsMapRef.current.clear();
  }, [state.sessionId]);

  // Start a new thread
  const startThread = useCallback(async (cwd?: string) => {
    if (!state.sessionId) return;

    try {
      setState(s => ({ ...s, isLoading: true, error: null }));
      itemsMapRef.current.clear();

      const response = await fetch('/api/codex/thread', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ sessionId: state.sessionId, cwd }),
      });

      const { error } = await response.json();
      if (error) throw new Error(error);

      setState(s => ({ ...s, isLoading: false, items: [] }));
    } catch (err) {
      setState(s => ({
        ...s,
        isLoading: false,
        error: err instanceof Error ? err.message : 'Failed to start thread',
      }));
    }
  }, [state.sessionId]);

  // Resume an existing thread
  const resumeThread = useCallback(async (threadId: string) => {
    if (!state.sessionId) return;

    try {
      setState(s => ({ ...s, isLoading: true, error: null }));
      itemsMapRef.current.clear();

      const response = await fetch('/api/codex/thread', {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ sessionId: state.sessionId, threadId }),
      });

      const { error } = await response.json();
      if (error) throw new Error(error);

      setState(s => ({ ...s, isLoading: false, items: [] }));
    } catch (err) {
      setState(s => ({
        ...s,
        isLoading: false,
        error: err instanceof Error ? err.message : 'Failed to resume thread',
      }));
    }
  }, [state.sessionId]);

  // Send a message
  const sendMessage = useCallback(async (message: string) => {
    if (!state.sessionId || !state.thread) return;

    try {
      setState(s => ({ ...s, error: null }));

      // Add user message to items immediately
      const userItem: ThreadItem = {
        type: 'userMessage',
        id: `user-${Date.now()}`,
        content: [{ type: 'text', text: message }],
      };
      itemsMapRef.current.set(userItem.id, userItem);
      setState(s => ({ ...s, items: Array.from(itemsMapRef.current.values()) }));

      const response = await fetch('/api/codex/turn', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          sessionId: state.sessionId,
          threadId: state.thread.id,
          input: message,
        }),
      });

      const { error } = await response.json();
      if (error) throw new Error(error);
    } catch (err) {
      setState(s => ({
        ...s,
        error: err instanceof Error ? err.message : 'Failed to send message',
      }));
    }
  }, [state.sessionId, state.thread]);

  // Interrupt current turn
  const interruptTurn = useCallback(async () => {
    if (!state.sessionId) return;

    try {
      await fetch(`/api/codex/turn?sessionId=${state.sessionId}`, { method: 'DELETE' });
    } catch (err) {
      console.error('[useCodex] Failed to interrupt:', err);
    }
  }, [state.sessionId]);

  // Load thread list
  const loadThreads = useCallback(async () => {
    if (!state.sessionId) return;

    try {
      const response = await fetch(`/api/codex/thread?sessionId=${state.sessionId}`);
      const { data, error } = await response.json();
      if (error) throw new Error(error);
      setState(s => ({ ...s, threads: data || [] }));
    } catch (err) {
      console.error('[useCodex] Failed to load threads:', err);
    }
  }, [state.sessionId]);

  // Cleanup on unmount
  useEffect(() => {
    return () => {
      if (eventSourceRef.current) {
        eventSourceRef.current.close();
      }
    };
  }, []);

  return {
    ...state,
    connect,
    disconnect,
    startThread,
    resumeThread,
    sendMessage,
    interruptTurn,
    loadThreads,
  };
}
