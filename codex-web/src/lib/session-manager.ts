import { CodexBridge } from './codex-bridge';
import { EventEmitter } from 'events';
import type { Thread, Turn, ThreadItem, CodexEvent, TurnStartParams } from '@/types/codex';

interface Session {
  id: string;
  bridge: CodexBridge;
  currentThread: Thread | null;
  currentTurn: Turn | null;
  items: Map<string, ThreadItem>;
  emitter: EventEmitter;
  createdAt: number;
  lastActivity: number;
}

/**
 * SessionManager handles multiple user sessions, each with their own
 * CodexBridge instance. This allows multiple concurrent users.
 */
class SessionManager {
  private sessions = new Map<string, Session>();
  private cleanupInterval: NodeJS.Timeout | null = null;
  private readonly SESSION_TIMEOUT = 30 * 60 * 1000; // 30 minutes

  constructor() {
    // Start cleanup interval
    this.cleanupInterval = setInterval(() => this.cleanupStaleSessions(), 60000);
  }

  /**
   * Create a new session
   */
  async createSession(sessionId: string): Promise<Session> {
    // Check if session already exists
    if (this.sessions.has(sessionId)) {
      return this.sessions.get(sessionId)!;
    }

    const bridge = new CodexBridge();
    const emitter = new EventEmitter();
    emitter.setMaxListeners(100); // Allow many SSE connections

    const session: Session = {
      id: sessionId,
      bridge,
      currentThread: null,
      currentTurn: null,
      items: new Map(),
      emitter,
      createdAt: Date.now(),
      lastActivity: Date.now(),
    };

    // Set up event forwarding from bridge to session emitter
    bridge.on('event', (event: CodexEvent) => {
      session.lastActivity = Date.now();

      // Update session state based on events
      this.handleEvent(session, event);

      // Forward to SSE listeners
      emitter.emit('event', event);
    });

    bridge.on('error', (error: Error) => {
      emitter.emit('error', error);
    });

    bridge.on('exit', (info: { code: number | null; signal: string | null }) => {
      emitter.emit('exit', info);
    });

    // Start the bridge
    await bridge.start();

    this.sessions.set(sessionId, session);
    return session;
  }

  /**
   * Get an existing session
   */
  getSession(sessionId: string): Session | undefined {
    const session = this.sessions.get(sessionId);
    if (session) {
      session.lastActivity = Date.now();
    }
    return session;
  }

  /**
   * Handle events and update session state
   */
  private handleEvent(session: Session, event: CodexEvent): void {
    switch (event.type) {
      case 'thread/started':
        session.currentThread = event.thread;
        break;
      case 'turn/started':
        session.currentTurn = event.turn;
        break;
      case 'turn/completed':
        session.currentTurn = event.turn;
        break;
      case 'item/started':
        session.items.set(event.item.id, event.item);
        break;
      case 'item/completed':
        session.items.set(event.item.id, event.item);
        break;
      case 'item/agentMessage/delta':
        // Update the agent message item with delta
        const agentItem = session.items.get(event.itemId);
        if (agentItem && agentItem.type === 'agentMessage') {
          agentItem.text = (agentItem.text || '') + event.delta;
        }
        break;
      case 'item/commandExecution/outputDelta':
        // Update command execution with output delta
        const cmdItem = session.items.get(event.itemId);
        if (cmdItem && cmdItem.type === 'commandExecution') {
          cmdItem.aggregatedOutput = (cmdItem.aggregatedOutput || '') + event.delta;
        }
        break;
    }
  }

  /**
   * Start a new thread in a session
   */
  async startThread(sessionId: string, params?: { model?: string; cwd?: string }): Promise<Thread> {
    const session = this.getSession(sessionId);
    if (!session) {
      throw new Error('Session not found');
    }
    session.items.clear();
    return await session.bridge.startThread(params);
  }

  /**
   * Resume an existing thread
   */
  async resumeThread(sessionId: string, threadId: string): Promise<Thread> {
    const session = this.getSession(sessionId);
    if (!session) {
      throw new Error('Session not found');
    }
    session.items.clear();
    return await session.bridge.resumeThread(threadId);
  }

  /**
   * Start a turn (send user message)
   */
  async startTurn(sessionId: string, params: TurnStartParams): Promise<Turn> {
    const session = this.getSession(sessionId);
    if (!session) {
      throw new Error('Session not found');
    }
    return await session.bridge.startTurn(params);
  }

  /**
   * Interrupt current turn
   */
  async interruptTurn(sessionId: string): Promise<void> {
    const session = this.getSession(sessionId);
    if (!session?.currentThread || !session?.currentTurn) {
      throw new Error('No active turn to interrupt');
    }
    await session.bridge.interruptTurn(session.currentThread.id, session.currentTurn.id);
  }

  /**
   * List threads
   */
  async listThreads(sessionId: string): Promise<{ data: Thread[]; nextCursor: string | null }> {
    const session = this.getSession(sessionId);
    if (!session) {
      throw new Error('Session not found');
    }
    return await session.bridge.listThreads();
  }

  /**
   * Delete a session
   */
  deleteSession(sessionId: string): void {
    const session = this.sessions.get(sessionId);
    if (session) {
      session.bridge.stop();
      session.emitter.removeAllListeners();
      this.sessions.delete(sessionId);
    }
  }

  /**
   * Clean up stale sessions
   */
  private cleanupStaleSessions(): void {
    const now = Date.now();
    for (const [id, session] of this.sessions) {
      if (now - session.lastActivity > this.SESSION_TIMEOUT) {
        console.log(`[SessionManager] Cleaning up stale session: ${id}`);
        this.deleteSession(id);
      }
    }
  }

  /**
   * Shutdown all sessions
   */
  shutdown(): void {
    if (this.cleanupInterval) {
      clearInterval(this.cleanupInterval);
    }
    for (const id of this.sessions.keys()) {
      this.deleteSession(id);
    }
  }
}

// Global singleton
const globalSessionManager = new SessionManager();

export function getSessionManager(): SessionManager {
  return globalSessionManager;
}

export type { Session };
