#!/bin/bash
# Codex Web UI Development Startup Script
# This script sets up the environment and starts the development server.

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CODEX_BINARY="$SCRIPT_DIR/codex-rs/target/debug/codex"
CODEX_WEB_DIR="$SCRIPT_DIR/codex-web"

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                    CODEX WEB UI                              ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

# Check if codex binary exists
if [ ! -f "$CODEX_BINARY" ]; then
    echo "❌ Codex binary not found at: $CODEX_BINARY"
    echo ""
    echo "Building codex binary..."
    echo ""
    cd "$SCRIPT_DIR/codex-rs"
    cargo build -p codex-cli
    echo ""
fi

echo "✓ Codex binary: $CODEX_BINARY"

# Check login status
echo ""
echo "Checking authentication status..."
if ! "$CODEX_BINARY" login status 2>/dev/null; then
    echo ""
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║                 AUTHENTICATION REQUIRED                      ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo ""
    echo "Codex requires authentication. Choose one of the following:"
    echo ""
    echo "  Option 1: Login with API Key"
    echo "    export OPENAI_API_KEY='your-api-key'"
    echo "    echo \$OPENAI_API_KEY | $CODEX_BINARY login --with-api-key"
    echo ""
    echo "  Option 2: Interactive Device Login"
    echo "    $CODEX_BINARY login --device-auth"
    echo ""
    echo "After authenticating, run this script again."
    exit 1
fi

echo "✓ Authentication: OK"

# Check if node_modules exists
if [ ! -d "$CODEX_WEB_DIR/node_modules" ]; then
    echo ""
    echo "Installing dependencies..."
    cd "$CODEX_WEB_DIR"
    npm install
fi

echo "✓ Dependencies: OK"
echo ""
echo "Starting development server..."
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Export the codex binary path
export CODEX_PATH="$CODEX_BINARY"

# Start the Next.js development server
cd "$CODEX_WEB_DIR"
exec npm run dev
