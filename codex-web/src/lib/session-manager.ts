import { CodexBridge } from './codex-bridge';
import { EventEmitter } from 'events';
import type { Thread, Turn, ThreadItem, CodexEvent, TurnStartParams } from '@/types/codex';

type SessionStatus = 'pending' | 'ready' | 'error';

interface Session {
  id: string;
  bridge: CodexBridge;
  currentThread: Thread | null;
  currentTurn: Turn | null;
  items: Map<string, ThreadItem>;
  emitter: EventEmitter;
  createdAt: number;
  lastActivity: number;
  status: SessionStatus;
  error?: string;
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
      status: 'pending',
    };

    // Store session immediately so events endpoint can find it
    this.sessions.set(sessionId, session);

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
    try {
      await bridge.start();
      session.status = 'ready';
      emitter.emit('ready');
    } catch (err) {
      session.status = 'error';
      session.error = err instanceof Error ? err.message : 'Failed to start bridge';
      emitter.emit('error', new Error(session.error));
    }

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
   * Check if a session exists and is valid (not stale)
   */
  hasSession(sessionId: string): boolean {
    const session = this.sessions.get(sessionId);
    if (!session) return false;

    // Check if session is stale
    const now = Date.now();
    if (now - session.lastActivity > this.SESSION_TIMEOUT) {
      this.deleteSession(sessionId);
      return false;
    }

    return true;
  }

  /**
   * Wait for a session to become ready
   */
  waitForReady(sessionId: string, timeout: number = 30000): Promise<Session> {
    return new Promise((resolve, reject) => {
      const session = this.sessions.get(sessionId);
      if (!session) {
        reject(new Error('Session not found'));
        return;
      }

      // Already ready
      if (session.status === 'ready') {
        resolve(session);
        return;
      }

      // Already errored
      if (session.status === 'error') {
        reject(new Error(session.error || 'Session initialization failed'));
        return;
      }

      // Wait for ready or error
      const timeoutId = setTimeout(() => {
        session.emitter.off('ready', onReady);
        session.emitter.off('error', onError);
        reject(new Error('Session initialization timeout'));
      }, timeout);

      const onReady = () => {
        clearTimeout(timeoutId);
        session.emitter.off('error', onError);
        resolve(session);
      };

      const onError = (err: Error) => {
        clearTimeout(timeoutId);
        session.emitter.off('ready', onReady);
        reject(err);
      };

      session.emitter.once('ready', onReady);
      session.emitter.once('error', onError);
    });
  }

  /**
   * Get a session and wait for it to be ready
   * Use this for all operations that require an active session
   */
  async getSessionReady(sessionId: string): Promise<Session> {
    const session = this.sessions.get(sessionId);
    if (!session) {
      throw new Error('Session not found');
    }

    if (session.status === 'ready') {
      session.lastActivity = Date.now();
      return session;
    }

    if (session.status === 'error') {
      throw new Error(session.error || 'Session initialization failed');
    }

    // Session is pending, wait for it
    return await this.waitForReady(sessionId);
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
    const session = await this.getSessionReady(sessionId);
    session.items.clear();
    return await session.bridge.startThread(params);
  }

  /**
   * Resume an existing thread
   */
  async resumeThread(sessionId: string, threadId: string): Promise<Thread> {
    const session = await this.getSessionReady(sessionId);
    session.items.clear();
    return await session.bridge.resumeThread(threadId);
  }

  /**
   * Start a turn (send user message)
   */
  async startTurn(sessionId: string, params: TurnStartParams): Promise<Turn> {
    const session = await this.getSessionReady(sessionId);
    return await session.bridge.startTurn(params);
  }

  /**
   * Interrupt current turn
   */
  async interruptTurn(sessionId: string): Promise<void> {
    const session = await this.getSessionReady(sessionId);
    if (!session.currentThread || !session.currentTurn) {
      throw new Error('No active turn to interrupt');
    }
    await session.bridge.interruptTurn(session.currentThread.id, session.currentTurn.id);
  }

  /**
   * List threads
   */
  async listThreads(sessionId: string): Promise<{ data: Thread[]; nextCursor: string | null }> {
    const session = await this.getSessionReady(sessionId);
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
   * Get auth status
   */
  async getAuthStatus(sessionId: string): Promise<{ status: string; account?: unknown }> {
    const session = await this.getSessionReady(sessionId);
    return await session.bridge.getAuthStatus();
  }

  /**
   * Login with API key
   */
  async loginApiKey(sessionId: string, apiKey: string): Promise<{ success: boolean; error?: string }> {
    const session = await this.getSessionReady(sessionId);
    return await session.bridge.loginApiKey(apiKey);
  }

  /**
   * Start device auth login
   */
  async loginDevice(sessionId: string): Promise<{ userCode: string; verificationUri: string; expiresIn: number }> {
    const session = await this.getSessionReady(sessionId);
    return await session.bridge.loginDevice();
  }

  /**
   * Cancel device auth login
   */
  async cancelLoginDevice(sessionId: string): Promise<void> {
    const session = await this.getSessionReady(sessionId);
    await session.bridge.cancelLoginDevice();
  }

  /**
   * Logout
   */
  async logout(sessionId: string): Promise<void> {
    const session = await this.getSessionReady(sessionId);
    await session.bridge.logout();
  }

  /**
   * Get account info
   */
  async getAccount(sessionId: string): Promise<unknown> {
    const session = await this.getSessionReady(sessionId);
    return await session.bridge.getAccount();
  }

  /**
   * List models
   */
  async listModels(sessionId: string): Promise<unknown> {
    const session = await this.getSessionReady(sessionId);
    return await session.bridge.listModels();
  }

  /**
   * Set default model
   */
  async setDefaultModel(sessionId: string, model: string): Promise<void> {
    const session = await this.getSessionReady(sessionId);
    await session.bridge.setDefaultModel(model);
  }

  /**
   * Read configuration
   */
  async readConfig(sessionId: string, key?: string): Promise<unknown> {
    const session = await this.getSessionReady(sessionId);
    return await session.bridge.readConfig(key);
  }

  /**
   * Write configuration
   */
  async writeConfig(sessionId: string, key: string, value: unknown): Promise<void> {
    const session = await this.getSessionReady(sessionId);
    await session.bridge.writeConfig(key, value);
  }

  /**
   * List skills
   */
  async listSkills(sessionId: string): Promise<unknown> {
    const session = await this.getSessionReady(sessionId);
    return await session.bridge.listSkills();
  }

  /**
   * Start code review
   */
  async startReview(sessionId: string, params: {
    target: 'uncommitted' | 'base' | 'commit' | 'custom';
    baseBranch?: string;
    commitSha?: string;
    instructions?: string;
  }): Promise<unknown> {
    const session = await this.getSessionReady(sessionId);
    return await session.bridge.startReview(params);
  }

  /**
   * Get MCP server status
   */
  async getMcpServerStatus(sessionId: string): Promise<unknown> {
    const session = await this.getSessionReady(sessionId);
    return await session.bridge.getMcpServerStatus();
  }

  /**
   * Fuzzy file search
   */
  async fuzzyFileSearch(sessionId: string, query: string, limit?: number): Promise<unknown> {
    const session = await this.getSessionReady(sessionId);
    return await session.bridge.fuzzyFileSearch(query, limit);
  }

  /**
   * Git diff to remote
   */
  async gitDiffToRemote(sessionId: string, branch?: string): Promise<unknown> {
    const session = await this.getSessionReady(sessionId);
    return await session.bridge.gitDiffToRemote(branch);
  }

  /**
   * Get user info
   */
  async getUserInfo(sessionId: string): Promise<unknown> {
    const session = await this.getSessionReady(sessionId);
    return await session.bridge.getUserInfo();
  }

  /**
   * Upload feedback
   */
  async uploadFeedback(sessionId: string, feedback: { type: string; message: string; includeLogs?: boolean }): Promise<void> {
    const session = await this.getSessionReady(sessionId);
    await session.bridge.uploadFeedback(feedback);
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

// Global singleton using globalThis to survive Next.js hot reloads
const globalForSessions = globalThis as unknown as {
  sessionManager: SessionManager | undefined;
};

if (!globalForSessions.sessionManager) {
  globalForSessions.sessionManager = new SessionManager();
}

export function getSessionManager(): SessionManager {
  return globalForSessions.sessionManager!;
}

export type { Session };
