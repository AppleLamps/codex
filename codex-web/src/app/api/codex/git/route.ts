import { NextRequest, NextResponse } from 'next/server';
import { getSessionManager } from '@/lib/session-manager';

/**
 * GET /api/codex/git
 * Get git diff to remote
 */
export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const sessionId = searchParams.get('sessionId');
    const branch = searchParams.get('branch') || undefined;

    if (!sessionId) {
      return NextResponse.json({ error: 'sessionId required' }, { status: 400 });
    }

    const manager = getSessionManager();
    const diff = await manager.gitDiffToRemote(sessionId, branch);

    return NextResponse.json(diff);
  } catch (error) {
    console.error('[API] Failed to get git diff:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Failed to get git diff' },
      { status: 500 }
    );
  }
}
