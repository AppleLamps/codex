// Codex App-Server Protocol Types
// Based on codex-rs/app-server/README.md

export interface ClientInfo {
  name: string;
  title: string;
  version: string;
}

// JSON-RPC Message Types
export interface JsonRpcRequest {
  method: string;
  id: number;
  params?: Record<string, unknown>;
}

export interface JsonRpcResponse {
  id: number;
  result?: unknown;
  error?: JsonRpcError;
}

export interface JsonRpcNotification {
  method: string;
  params: Record<string, unknown>;
}

export interface JsonRpcError {
  code: number;
  message: string;
  data?: unknown;
}

// Thread Types
export interface Thread {
  id: string;
  preview: string;
  modelProvider: string;
  createdAt: number;
}

export interface Turn {
  id: string;
  status: 'inProgress' | 'completed' | 'interrupted' | 'failed';
  items: ThreadItem[];
  error?: TurnError;
}

export interface TurnError {
  message: string;
  codexErrorInfo?: string;
  additionalDetails?: string;
}

// Input Types
export type UserInput =
  | { type: 'text'; text: string }
  | { type: 'image'; url: string }
  | { type: 'localImage'; path: string };

// Thread Items
export type ThreadItem =
  | UserMessageItem
  | AgentMessageItem
  | ReasoningItem
  | CommandExecutionItem
  | FileChangeItem
  | McpToolCallItem
  | WebSearchItem
  | TodoListItem
  | ErrorItem;

export interface UserMessageItem {
  type: 'userMessage';
  id: string;
  content: UserInput[];
}

export interface AgentMessageItem {
  type: 'agentMessage';
  id: string;
  text: string;
}

export interface ReasoningItem {
  type: 'reasoning';
  id: string;
  summary?: string;
  content?: string;
}

export interface CommandExecutionItem {
  type: 'commandExecution';
  id: string;
  command: string;
  cwd?: string;
  status: 'inProgress' | 'completed' | 'failed' | 'declined';
  aggregatedOutput?: string;
  exitCode?: number;
  durationMs?: number;
}

export interface FileChangeItem {
  type: 'fileChange';
  id: string;
  changes: FileChange[];
  status: 'inProgress' | 'completed' | 'failed' | 'declined';
}

export interface FileChange {
  path: string;
  kind: 'add' | 'delete' | 'update';
  diff?: string;
}

export interface McpToolCallItem {
  type: 'mcpToolCall';
  id: string;
  server: string;
  tool: string;
  status: 'inProgress' | 'completed' | 'failed';
  arguments?: unknown;
  result?: unknown;
  error?: { message: string };
}

export interface WebSearchItem {
  type: 'webSearch';
  id: string;
  query: string;
}

export interface TodoListItem {
  type: 'todoList';
  id: string;
  items: { text: string; completed: boolean }[];
}

export interface ErrorItem {
  type: 'error';
  id: string;
  message: string;
}

// Sandbox Policy
export type SandboxPolicy =
  | { type: 'readOnly' }
  | { type: 'workspaceWrite'; writableRoots?: string[]; networkAccess?: boolean }
  | { type: 'dangerFullAccess' }
  | { type: 'externalSandbox'; networkAccess?: 'restricted' | 'enabled' };

// Turn Start Parameters
export interface TurnStartParams {
  threadId: string;
  input: UserInput[];
  cwd?: string;
  approvalPolicy?: 'never' | 'onRequest' | 'onFailure' | 'unlessTrusted';
  sandboxPolicy?: SandboxPolicy;
  model?: string;
  effort?: 'minimal' | 'low' | 'medium' | 'high' | 'xhigh';
}

// Thread Start Parameters
export interface ThreadStartParams {
  model?: string;
  cwd?: string;
  approvalPolicy?: 'never' | 'onRequest' | 'onFailure' | 'unlessTrusted';
  sandbox?: 'readOnly' | 'workspaceWrite' | 'dangerFullAccess';
}

// Events from server
export type CodexEvent =
  | { type: 'thread/started'; thread: Thread }
  | { type: 'turn/started'; turn: Turn }
  | { type: 'turn/completed'; turn: Turn }
  | { type: 'item/started'; item: ThreadItem }
  | { type: 'item/completed'; item: ThreadItem }
  | { type: 'item/agentMessage/delta'; itemId: string; delta: string }
  | { type: 'item/commandExecution/outputDelta'; itemId: string; delta: string }
  | { type: 'item/reasoning/summaryTextDelta'; itemId: string; delta: string }
  | { type: 'error'; error: TurnError }
  | { type: 'connected' }
  | { type: 'exit'; code: number | null; signal: string | null }
  | { type: 'account/updated'; account: unknown };

// WebSocket message from client to server
export interface WsClientMessage {
  type: 'initialize' | 'thread/start' | 'thread/list' | 'turn/start' | 'turn/interrupt';
  params?: Record<string, unknown>;
}

// WebSocket message from server to client
export interface WsServerMessage {
  type: 'initialized' | 'event' | 'error' | 'result';
  data?: unknown;
  error?: string;
}
