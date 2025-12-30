# Codex Web UI Documentation

This document provides context for AI agents and developers working on the codex-web application.

---

## Project Overview

The codex-web is a Next.js web interface for the Codex CLI tool. It provides a ChatGPT-like interface for interacting with the Codex agent.

**Location:** `codex-web/`

---

## Tech Stack

| Technology | Version | Purpose |
|------------|---------|---------|
| Next.js | 15.5.9 | React framework with App Router |
| React | 18+ | UI library |
| TypeScript | - | Type safety |
| Tailwind CSS | 3.4.17 | Styling |
| Lucide React | - | Icons |
| clsx | - | Conditional classnames |

---

## Project Structure

```
codex-web/
├── src/
│   ├── app/                    # Next.js App Router
│   │   ├── api/codex/          # API routes
│   │   │   ├── auth/           # Authentication endpoints
│   │   │   ├── events/         # SSE event stream
│   │   │   ├── session/        # Session management
│   │   │   ├── thread/         # Thread operations
│   │   │   ├── turn/           # Turn (message) operations
│   │   │   ├── models/         # Model listing/selection
│   │   │   └── ...             # Other endpoints
│   │   ├── globals.css         # Global styles + CSS variables
│   │   ├── layout.tsx          # Root layout with ThemeProvider
│   │   └── page.tsx            # Main chat interface
│   ├── components/             # React components
│   │   ├── Sidebar.tsx         # Left sidebar with sessions
│   │   ├── ChatThread.tsx      # Message display area
│   │   ├── MessageItem.tsx     # Individual message rendering
│   │   ├── InputBox.tsx        # Message input
│   │   ├── LoginDialog.tsx     # Auth dialog (API key / device auth)
│   │   ├── SettingsPanel.tsx   # Settings slide-out
│   │   └── ...                 # Other components
│   ├── lib/
│   │   ├── hooks/
│   │   │   └── useCodex.ts     # Main hook for Codex interaction
│   │   ├── codex-bridge.ts     # Bridge to Codex CLI subprocess
│   │   ├── session-manager.ts  # Server-side session management
│   │   └── theme-context.tsx   # Theme state (light/dark)
│   └── types/
│       └── codex.ts            # TypeScript type definitions
├── tailwind.config.js          # Tailwind configuration
├── next.config.js              # Next.js configuration
└── package.json
```

---

## Key Systems

### 1. Session Management

Sessions connect the web UI to Codex CLI subprocesses.

**How it works:**
- Each session has a unique UUID stored in `localStorage` (`codex-session-id`)
- Sessions are managed server-side in `session-manager.ts` (in-memory Map)
- Each session spawns a `codex app-server` subprocess via `CodexBridge`
- Sessions timeout after 30 minutes of inactivity

**Session flow:**
```
Page Load → useCodex.connect() → POST /api/codex/session
         → Check localStorage for existing sessionId
         → If exists & valid on server → reuse session
         → If not → create new session, store in localStorage
         → Connect to SSE at /api/codex/events?sessionId=xxx
```

**Important files:**
- `src/lib/hooks/useCodex.ts` - Client-side session handling
- `src/lib/session-manager.ts` - Server-side session storage
- `src/app/api/codex/session/route.ts` - Session API endpoint

**Known limitation:** Sessions are in-memory only. Server restart loses all sessions.

---

### 2. Authentication

Authentication is handled through the Codex CLI, not the web app directly.

**Methods:**
1. **API Key** - User enters Anthropic API key
2. **Device Auth** - OAuth-like flow with verification code

**How it works:**
- Auth state is tied to the session/subprocess
- When you authenticate, credentials are stored in the Codex CLI process
- New session = new subprocess = no authentication

**Auth persistence:**
- Authentication persists as long as the session persists
- Session persists via localStorage (survives page refresh)
- Explicit logout clears both server session and localStorage

**Important files:**
- `src/app/api/codex/auth/route.ts` - Auth API endpoints
- `src/components/LoginDialog.tsx` - Login UI

---

### 3. Theming System

The app supports light mode (default) and dark mode.

**Implementation:**
- CSS variables defined in `globals.css` (`:root` for light, `.dark` for dark)
- Tailwind colors reference CSS variables (`var(--codex-xxx)`)
- Theme state managed in `theme-context.tsx`
- Theme preference stored in localStorage (`codex-theme`)

**Color tokens:**
| Token | Light Mode | Dark Mode | Usage |
|-------|------------|-----------|-------|
| `--codex-bg` | #ffffff | #0a0a0a | Main background |
| `--codex-surface` | #f7f7f8 | #141414 | Elevated surfaces |
| `--codex-border` | #e5e5e5 | #262626 | Borders |
| `--codex-text` | #1a1a1a | #fafafa | Primary text |
| `--codex-muted` | #6b6b6b | #a1a1a1 | Secondary text |
| `--codex-hover` | #ececec | #1f1f1f | Hover states |
| `--codex-accent` | #000000 | #ffffff | Accent/buttons |

**Semantic colors (static):**
- `codex-success`: #22c55e (green)
- `codex-error`: #ef4444 (red)
- `codex-warning`: #f59e0b (amber)

**Important files:**
- `src/app/globals.css` - CSS variable definitions
- `src/lib/theme-context.tsx` - Theme state provider
- `tailwind.config.js` - Tailwind color mappings

---

### 4. Real-time Communication (SSE)

The app uses Server-Sent Events for real-time updates from Codex.

**How it works:**
1. Client connects to `/api/codex/events?sessionId=xxx`
2. Server keeps connection open, streams events as they occur
3. Events include: thread updates, turn progress, message deltas, errors

**Event types:**
- `connected` - Initial connection confirmation
- `thread/started` - New thread created
- `turn/started` / `turn/completed` - Turn lifecycle
- `item/started` / `item/completed` - Message items
- `item/agentMessage/delta` - Streaming text from agent
- `item/commandExecution/outputDelta` - Command output streaming
- `error` - Error events
- `exit` - Session ended

**Important files:**
- `src/app/api/codex/events/route.ts` - SSE endpoint
- `src/lib/hooks/useCodex.ts` - Client-side event handling

---

## Component Patterns

### Button Styling Convention

Primary/accent buttons use theme-aware colors:
```tsx
className="bg-codex-accent text-codex-bg hover:opacity-80"
```

Secondary buttons:
```tsx
className="border border-codex-border text-codex-text hover:bg-codex-hover"
```

### Icon Usage

All icons come from `lucide-react`. Common icons:
- `Sun` / `Moon` - Theme toggle
- `Plus` - New session
- `Send` - Send message
- `AlertCircle` - Warnings/errors
- `Loader2` - Loading spinner (use with `className="animate-spin"`)

---

## API Routes

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/codex/session` | POST | Create/resume session |
| `/api/codex/session` | DELETE | Delete session |
| `/api/codex/events` | GET | SSE event stream |
| `/api/codex/auth` | GET | Check auth status |
| `/api/codex/auth` | POST | Login (API key or device) |
| `/api/codex/auth` | DELETE | Logout |
| `/api/codex/thread` | POST | Start new thread |
| `/api/codex/thread` | PUT | Resume thread |
| `/api/codex/thread` | GET | List threads |
| `/api/codex/turn` | POST | Send message |
| `/api/codex/turn` | DELETE | Interrupt turn |
| `/api/codex/models` | GET | List available models |
| `/api/codex/models` | PUT | Set default model |

All endpoints require `sessionId` (query param or body).

---

## Development

### Running Locally

```bash
cd codex-web
npm install
npm run dev
```

Or use the PowerShell script from the repo root:
```powershell
.\start-dev.ps1
```

### Known Issues / Warnings

1. **Next.js config warning**: `serverComponentsExternalPackages` moved to `serverExternalPackages`
2. **Multiple lockfiles**: There are lockfiles at different levels - may cause warnings
3. **Model refresh error**: The models endpoint may return 404 from Anthropic API (non-blocking)

---

## Recent Changes

### Session Persistence (Dec 2024)
- Sessions now persist across page refreshes via localStorage
- Session ID stored in `codex-session-id` key
- Server reuses existing sessions if valid

### Light Mode (Dec 2024)
- Light mode is now the default theme
- Theme toggle button in header (sun/moon icon)
- Theme preference persists in localStorage (`codex-theme`)
- All UI elements are theme-aware

---

## Tips for AI Agents

1. **Always check session state** - Many operations require an active, authenticated session

2. **CSS variables over hardcoded colors** - Use `codex-*` Tailwind classes, not `bg-white`/`text-black`

3. **Theme-aware accent colors** - For buttons that should be dark in light mode and light in dark mode:
   ```tsx
   bg-codex-accent text-codex-bg
   ```

4. **Session timeout** - Sessions expire after 30 minutes of inactivity. If operations fail, the session may have expired.

5. **Auth is session-bound** - Authentication lives in the Codex subprocess. New session = need to re-authenticate.

6. **No database** - Everything is in-memory or localStorage. Server restart = clean slate.

7. **Check types** - Type definitions are in `src/types/codex.ts`

8. **The bridge pattern** - `CodexBridge` spawns and communicates with the actual Codex CLI. Don't modify the bridge lightly.
