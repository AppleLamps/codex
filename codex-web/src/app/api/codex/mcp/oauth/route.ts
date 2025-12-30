import { NextRequest, NextResponse } from 'next/server';
import { getSessionManager } from '@/lib/session-manager';

/**
 * POST /api/codex/mcp/oauth
 * Initiate MCP OAuth login for a specific server
 */
export async function POST(request: NextRequest) {
  try {
    const { sessionId, serverName } = await request.json();

    if (!sessionId) {
      return NextResponse.json({ error: 'sessionId required' }, { status: 400 });
    }

    if (!serverName) {
      return NextResponse.json({ error: 'serverName required' }, { status: 400 });
    }

    const manager = getSessionManager();
    const result = await manager.mcpOAuthLogin(sessionId, serverName);

    return NextResponse.json(result);
  } catch (error) {
    console.error('[API] Failed to initiate MCP OAuth:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Failed to initiate MCP OAuth' },
      { status: 500 }
    );
  }
}
