import { NextRequest, NextResponse } from 'next/server';
import { getSessionManager } from '@/lib/session-manager';

/**
 * GET /api/codex/ratelimits
 * Get rate limits for the current account
 */
export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const sessionId = searchParams.get('sessionId');

    if (!sessionId) {
      return NextResponse.json({ error: 'sessionId required' }, { status: 400 });
    }

    const manager = getSessionManager();
    const rateLimits = await manager.getRateLimits(sessionId);

    return NextResponse.json(rateLimits);
  } catch (error) {
    console.error('[API] Failed to get rate limits:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Failed to get rate limits' },
      { status: 500 }
    );
  }
}
