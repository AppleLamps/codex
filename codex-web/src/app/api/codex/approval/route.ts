import { NextRequest, NextResponse } from 'next/server';
import { getSessionManager } from '@/lib/session-manager';

/**
 * POST /api/codex/approval
 * Respond to an approval request
 */
export async function POST(request: NextRequest) {
  try {
    const { sessionId, itemId, approved } = await request.json();

    if (!sessionId) {
      return NextResponse.json({ error: 'sessionId required' }, { status: 400 });
    }

    if (!itemId) {
      return NextResponse.json({ error: 'itemId required' }, { status: 400 });
    }

    if (typeof approved !== 'boolean') {
      return NextResponse.json({ error: 'approved (boolean) required' }, { status: 400 });
    }

    const manager = getSessionManager();
    await manager.respondToApproval(sessionId, itemId, approved);

    return NextResponse.json({ message: 'Approval response sent' });
  } catch (error) {
    console.error('[API] Failed to respond to approval:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Failed to respond to approval' },
      { status: 500 }
    );
  }
}
