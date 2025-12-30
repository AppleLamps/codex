import { NextRequest, NextResponse } from 'next/server';
import { getSessionManager } from '@/lib/session-manager';

/**
 * GET /api/codex/user
 * Get user info
 */
export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const sessionId = searchParams.get('sessionId');

    if (!sessionId) {
      return NextResponse.json({ error: 'sessionId required' }, { status: 400 });
    }

    const manager = getSessionManager();
    const userInfo = await manager.getUserInfo(sessionId);

    return NextResponse.json(userInfo);
  } catch (error) {
    console.error('[API] Failed to get user info:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Failed to get user info' },
      { status: 500 }
    );
  }
}
