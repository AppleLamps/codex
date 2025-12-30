import { NextRequest, NextResponse } from 'next/server';
import { getSessionManager } from '@/lib/session-manager';

/**
 * GET /api/codex/account
 * Get account info
 */
export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const sessionId = searchParams.get('sessionId');

    if (!sessionId) {
      return NextResponse.json({ error: 'sessionId required' }, { status: 400 });
    }

    const manager = getSessionManager();
    const account = await manager.getAccount(sessionId);

    return NextResponse.json(account);
  } catch (error) {
    console.error('[API] Failed to get account:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Failed to get account' },
      { status: 500 }
    );
  }
}
