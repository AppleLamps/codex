import { ChildProcess, spawn } from 'child_process';
import { createInterface, Interface } from 'readline';
import { EventEmitter } from 'events';
import type {
  JsonRpcRequest,
  JsonRpcResponse,
  JsonRpcNotification,
  Thread,
  Turn,
  ThreadItem,
  TurnStartParams,
  ThreadStartParams,
  CodexEvent,
} from '@/types/codex';

/**
 * CodexBridge manages the codex app-server process and provides
 * a high-level API for communicating with it via JSON-RPC.
 */
export class CodexBridge extends EventEmitter {
  private process: ChildProcess | null = null;
  private readline: Interface | null = null;
  private requestId = 0;
  private pendingRequests = new Map<number, {
    resolve: (value: unknown) => void;
    reject: (error: Error) => void;
  }>();
  private initialized = false;
  private codexPath: string;

  constructor(codexPath?: string) {
    super();
    // Default to 'codex' in PATH, or override with explicit path
    this.codexPath = codexPath || process.env.CODEX_PATH || 'codex';
  }

  /**
   * Spawn the codex app-server process and set up communication
   */
  async start(): Promise<void> {
    if (this.process) {
      throw new Error('CodexBridge already started');
    }

    return new Promise((resolve, reject) => {
      try {
        this.process = spawn(this.codexPath, ['app-server'], {
          stdio: ['pipe', 'pipe', 'pipe'],
          env: {
            ...process.env,
            // Ensure we don't inherit any TTY settings
            TERM: 'dumb',
          },
        });

        this.process.on('error', (err) => {
          this.emit('error', err);
          reject(err);
        });

        this.process.on('exit', (code, signal) => {
          this.emit('exit', { code, signal });
          this.cleanup();
        });

        // Set up line-by-line reading from stdout
        if (this.process.stdout) {
          this.readline = createInterface({
            input: this.process.stdout,
            crlfDelay: Infinity,
          });

          this.readline.on('line', (line) => {
            this.handleLine(line);
          });
        }

        // Log stderr for debugging
        if (this.process.stderr) {
          this.process.stderr.on('data', (data) => {
            console.error('[codex stderr]', data.toString());
          });
        }

        // Initialize the connection
        this.initialize()
          .then(() => {
            this.initialized = true;
            resolve();
          })
          .catch(reject);
      } catch (err) {
        reject(err);
      }
    });
  }

  /**
   * Send initialize request per the protocol
   */
  private async initialize(): Promise<void> {
    const result = await this.request('initialize', {
      clientInfo: {
        name: 'codex-web',
        title: 'Codex Web UI',
        version: '0.1.0',
      },
    });

    // Send initialized notification
    this.notify('initialized', {});

    return result as void;
  }

  /**
   * Send a JSON-RPC request and wait for response
   */
  private request(method: string, params?: object): Promise<unknown> {
    return new Promise((resolve, reject) => {
      const id = ++this.requestId;
      const request: JsonRpcRequest = { method, id };
      if (params) {
        request.params = params as Record<string, unknown>;
      }

      this.pendingRequests.set(id, { resolve, reject });
      this.send(request);

      // Timeout after 30 seconds
      setTimeout(() => {
        if (this.pendingRequests.has(id)) {
          this.pendingRequests.delete(id);
          reject(new Error(`Request ${method} timed out`));
        }
      }, 30000);
    });
  }

  /**
   * Send a JSON-RPC notification (no response expected)
   */
  private notify(method: string, params: Record<string, unknown>): void {
    const notification: JsonRpcNotification = { method, params };
    this.send(notification);
  }

  /**
   * Write a message to the process stdin
   */
  private send(message: object): void {
    if (!this.process?.stdin?.writable) {
      throw new Error('Process not running or stdin not writable');
    }
    const line = JSON.stringify(message);
    this.process.stdin.write(line + '\n');
  }

  /**
   * Handle a line of output from the process
   */
  private handleLine(line: string): void {
    if (!line.trim()) return;

    try {
      const message = JSON.parse(line);

      // Check if it's a response (has 'id')
      if ('id' in message && typeof message.id === 'number') {
        const pending = this.pendingRequests.get(message.id);
        if (pending) {
          this.pendingRequests.delete(message.id);
          if (message.error) {
            pending.reject(new Error(message.error.message || 'Unknown error'));
          } else {
            pending.resolve(message.result);
          }
        }
        return;
      }

      // It's a notification - emit as an event
      if ('method' in message) {
        this.handleNotification(message as JsonRpcNotification);
      }
    } catch (err) {
      console.error('[codex] Failed to parse line:', line, err);
    }
  }

  /**
   * Handle incoming notifications from the server
   */
  private handleNotification(notification: JsonRpcNotification): void {
    const { method, params } = notification;

    // Map notification methods to event types
    const eventMap: Record<string, string> = {
      'thread/started': 'thread/started',
      'turn/started': 'turn/started',
      'turn/completed': 'turn/completed',
      'item/started': 'item/started',
      'item/completed': 'item/completed',
      'item/agentMessage/delta': 'item/agentMessage/delta',
      'item/commandExecution/outputDelta': 'item/commandExecution/outputDelta',
      'item/reasoning/summaryTextDelta': 'item/reasoning/summaryTextDelta',
      'item/reasoning/textDelta': 'item/reasoning/textDelta',
      'error': 'error',
      'account/updated': 'account/updated',
    };

    const eventType = eventMap[method] || method;
    const event: CodexEvent = {
      type: eventType as CodexEvent['type'],
      ...params,
    } as CodexEvent;

    this.emit('event', event);
    this.emit(eventType, params);
  }

  /**
   * Start a new thread
   */
  async startThread(params?: ThreadStartParams): Promise<Thread> {
    if (!this.initialized) {
      throw new Error('CodexBridge not initialized');
    }
    const result = await this.request('thread/start', params || {}) as { thread: Thread };
    return result.thread;
  }

  /**
   * Resume an existing thread
   */
  async resumeThread(threadId: string): Promise<Thread> {
    if (!this.initialized) {
      throw new Error('CodexBridge not initialized');
    }
    const result = await this.request('thread/resume', { threadId }) as { thread: Thread };
    return result.thread;
  }

  /**
   * List all threads
   */
  async listThreads(cursor?: string, limit?: number): Promise<{ data: Thread[]; nextCursor: string | null }> {
    if (!this.initialized) {
      throw new Error('CodexBridge not initialized');
    }
    const params: Record<string, unknown> = {};
    if (cursor) params.cursor = cursor;
    if (limit) params.limit = limit;
    return await this.request('thread/list', params) as { data: Thread[]; nextCursor: string | null };
  }

  /**
   * Archive a thread
   */
  async archiveThread(threadId: string): Promise<void> {
    if (!this.initialized) {
      throw new Error('CodexBridge not initialized');
    }
    await this.request('thread/archive', { threadId });
  }

  /**
   * Start a new turn (send user input)
   */
  async startTurn(params: TurnStartParams): Promise<Turn> {
    if (!this.initialized) {
      throw new Error('CodexBridge not initialized');
    }
    const result = await this.request('turn/start', params) as { turn: Turn };
    return result.turn;
  }

  /**
   * Interrupt an active turn
   */
  async interruptTurn(threadId: string, turnId: string): Promise<void> {
    if (!this.initialized) {
      throw new Error('CodexBridge not initialized');
    }
    await this.request('turn/interrupt', { threadId, turnId });
  }

  /**
   * Get account info
   */
  async getAccount(): Promise<unknown> {
    if (!this.initialized) {
      throw new Error('CodexBridge not initialized');
    }
    return await this.request('account/read', { refreshToken: false });
  }

  /**
   * List available models
   */
  async listModels(): Promise<unknown> {
    if (!this.initialized) {
      throw new Error('CodexBridge not initialized');
    }
    return await this.request('model/list', {});
  }

  /**
   * Set default model
   */
  async setDefaultModel(model: string): Promise<void> {
    if (!this.initialized) {
      throw new Error('CodexBridge not initialized');
    }
    await this.request('setDefaultModel', { model });
  }

  /**
   * Get authentication status
   */
  async getAuthStatus(): Promise<{ status: string; account?: unknown }> {
    if (!this.initialized) {
      throw new Error('CodexBridge not initialized');
    }
    return await this.request('getAuthStatus', {}) as { status: string; account?: unknown };
  }

  /**
   * Login with API key
   */
  async loginApiKey(apiKey: string): Promise<{ success: boolean; error?: string }> {
    if (!this.initialized) {
      throw new Error('CodexBridge not initialized');
    }
    try {
      await this.request('loginApiKey', { apiKey });
      return { success: true };
    } catch (err) {
      return { success: false, error: err instanceof Error ? err.message : 'Login failed' };
    }
  }

  /**
   * Start device auth login flow
   */
  async loginDevice(): Promise<{ userCode: string; verificationUri: string; expiresIn: number }> {
    if (!this.initialized) {
      throw new Error('CodexBridge not initialized');
    }
    const result = await this.request('loginChatGpt', {}) as Record<string, unknown>;
    // Handle both camelCase and snake_case response formats
    return {
      userCode: (result.userCode || result.user_code || '') as string,
      verificationUri: (result.verificationUri || result.verification_uri || '') as string,
      expiresIn: (result.expiresIn || result.expires_in || 0) as number,
    };
  }

  /**
   * Cancel ongoing device auth login
   */
  async cancelLoginDevice(): Promise<void> {
    if (!this.initialized) {
      throw new Error('CodexBridge not initialized');
    }
    await this.request('cancelLoginChatGpt', {});
  }

  /**
   * Logout
   */
  async logout(): Promise<void> {
    if (!this.initialized) {
      throw new Error('CodexBridge not initialized');
    }
    await this.request('account/logout', {});
  }

  /**
   * Read configuration
   */
  async readConfig(key?: string): Promise<unknown> {
    if (!this.initialized) {
      throw new Error('CodexBridge not initialized');
    }
    const params = key ? { key } : {};
    return await this.request('config/read', params);
  }

  /**
   * Write configuration
   */
  async writeConfig(key: string, value: unknown): Promise<void> {
    if (!this.initialized) {
      throw new Error('CodexBridge not initialized');
    }
    await this.request('config/value/write', { key, value });
  }

  /**
   * List available skills
   */
  async listSkills(): Promise<unknown> {
    if (!this.initialized) {
      throw new Error('CodexBridge not initialized');
    }
    return await this.request('skills/list', {});
  }

  /**
   * Start a code review
   */
  async startReview(params: {
    target: 'uncommitted' | 'base' | 'commit' | 'custom';
    baseBranch?: string;
    commitSha?: string;
    instructions?: string;
  }): Promise<unknown> {
    if (!this.initialized) {
      throw new Error('CodexBridge not initialized');
    }
    return await this.request('review/start', params);
  }

  /**
   * Get MCP server status
   */
  async getMcpServerStatus(): Promise<unknown> {
    if (!this.initialized) {
      throw new Error('CodexBridge not initialized');
    }
    return await this.request('mcpServerStatus/list', {});
  }

  /**
   * Fuzzy file search
   */
  async fuzzyFileSearch(query: string, limit?: number): Promise<unknown> {
    if (!this.initialized) {
      throw new Error('CodexBridge not initialized');
    }
    const params: Record<string, unknown> = { query };
    if (limit) params.limit = limit;
    return await this.request('fuzzyFileSearch', params);
  }

  /**
   * Get git diff to remote
   */
  async gitDiffToRemote(branch?: string): Promise<unknown> {
    if (!this.initialized) {
      throw new Error('CodexBridge not initialized');
    }
    const params = branch ? { branch } : {};
    return await this.request('gitDiffToRemote', params);
  }

  /**
   * Get user info
   */
  async getUserInfo(): Promise<unknown> {
    if (!this.initialized) {
      throw new Error('CodexBridge not initialized');
    }
    return await this.request('userInfo', {});
  }

  /**
   * Upload feedback
   */
  async uploadFeedback(feedback: { type: string; message: string; includeLogs?: boolean }): Promise<void> {
    if (!this.initialized) {
      throw new Error('CodexBridge not initialized');
    }
    await this.request('feedback/upload', feedback);
  }

  /**
   * Get rate limits
   */
  async getRateLimits(): Promise<unknown> {
    if (!this.initialized) {
      throw new Error('CodexBridge not initialized');
    }
    return await this.request('account/rateLimits/read', {});
  }

  /**
   * Start MCP OAuth login
   */
  async mcpOAuthLogin(serverName: string): Promise<unknown> {
    if (!this.initialized) {
      throw new Error('CodexBridge not initialized');
    }
    return await this.request('mcpServer/oauth/login', { serverName });
  }

  /**
   * Respond to approval request
   */
  async respondToApproval(itemId: string, approved: boolean): Promise<void> {
    if (!this.initialized) {
      throw new Error('CodexBridge not initialized');
    }
    await this.request('item/approval/respond', { itemId, approved });
  }

  /**
   * Clean up resources
   */
  private cleanup(): void {
    this.readline?.close();
    this.readline = null;
    this.process = null;
    this.initialized = false;
    this.pendingRequests.clear();
  }

  /**
   * Stop the bridge and kill the process
   */
  stop(): void {
    if (this.process) {
      this.process.kill();
      this.cleanup();
    }
  }

  /**
   * Check if the bridge is running
   */
  isRunning(): boolean {
    return this.process !== null && this.initialized;
  }
}

// Singleton instance for the application
let bridgeInstance: CodexBridge | null = null;

export function getCodexBridge(): CodexBridge {
  if (!bridgeInstance) {
    bridgeInstance = new CodexBridge();
  }
  return bridgeInstance;
}

export function resetCodexBridge(): void {
  if (bridgeInstance) {
    bridgeInstance.stop();
    bridgeInstance = null;
  }
}
