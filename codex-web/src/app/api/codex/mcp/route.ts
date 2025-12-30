import { NextRequest, NextResponse } from 'next/server';
import { getSessionManager } from '@/lib/session-manager';

/**
 * GET /api/codex/mcp
 * Get MCP server status
 */
export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const sessionId = searchParams.get('sessionId');

    if (!sessionId) {
      return NextResponse.json({ error: 'sessionId required' }, { status: 400 });
    }

    const manager = getSessionManager();
    const status = await manager.getMcpServerStatus(sessionId);

    return NextResponse.json(status);
  } catch (error) {
    console.error('[API] Failed to get MCP status:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Failed to get MCP status' },
      { status: 500 }
    );
  }
}
