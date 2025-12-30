# Codex Web UI

A modern web interface for the OpenAI Codex CLI, built with Next.js and React.

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Browser                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    React Frontend                         │   │
│  │  - Sidebar (session list)                                │   │
│  │  - ChatThread (message display)                          │   │
│  │  - InputBox (user input)                                 │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              │                                   │
│                    REST API + SSE                               │
└──────────────────────────────┼──────────────────────────────────┘
                               │
┌──────────────────────────────┼──────────────────────────────────┐
│                    Next.js Server                                │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                  Session Manager                          │   │
│  │  - Manages multiple user sessions                        │   │
│  │  - Routes API requests to correct bridge                 │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              │                                   │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                   Codex Bridge                            │   │
│  │  - Spawns codex app-server process                       │   │
│  │  - JSON-RPC communication over stdio                     │   │
│  │  - Event forwarding to SSE clients                       │   │
│  └─────────────────────────────────────────────────────────┘   │
└──────────────────────────────┼──────────────────────────────────┘
                               │
                          stdin/stdout
                               │
┌──────────────────────────────┼──────────────────────────────────┐
│                    codex app-server                              │
│  - Thread management                                             │
│  - Turn execution                                                │
│  - Tool calls (commands, file edits, etc.)                      │
└─────────────────────────────────────────────────────────────────┘
```

## Prerequisites

- Node.js 18+
- The `codex` binary must be installed and available in your PATH
  - Build it from the monorepo: `cargo build --release -p codex`
  - Or set `CODEX_PATH` environment variable to the binary location

## Getting Started

1. Install dependencies:

```bash
cd codex-web
npm install
```

2. Run the development server:

```bash
npm run dev
```

3. Open [http://localhost:3000](http://localhost:3000) in your browser.

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `CODEX_PATH` | Path to the codex binary | `codex` (uses PATH) |

## API Endpoints

### Sessions

- `POST /api/codex/session` - Create a new session
- `DELETE /api/codex/session?sessionId=...` - Delete a session

### Threads

- `POST /api/codex/thread` - Start a new thread
- `GET /api/codex/thread?sessionId=...` - List all threads
- `PUT /api/codex/thread` - Resume an existing thread

### Turns

- `POST /api/codex/turn` - Start a new turn (send message)
- `DELETE /api/codex/turn?sessionId=...` - Interrupt current turn

### Events

- `GET /api/codex/events?sessionId=...` - Server-Sent Events stream

## Features

- Real-time streaming responses via SSE
- Session persistence (threads are saved locally)
- Command execution with output streaming
- File change visualization with diffs
- Reasoning/thinking display
- Dark theme with black accents

## Development

```bash
# Run development server
npm run dev

# Build for production
npm run build

# Start production server
npm start

# Lint
npm run lint
```

## Tech Stack

- **Framework**: Next.js 15 (App Router)
- **UI**: React 19 + Tailwind CSS
- **Icons**: Lucide React
- **Communication**: Server-Sent Events (SSE)
- **Process Management**: Node.js child_process
