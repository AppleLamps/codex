import { NextRequest, NextResponse } from 'next/server';
import { getSessionManager } from '@/lib/session-manager';

/**
 * POST /api/codex/feedback
 * Upload feedback
 */
export async function POST(request: NextRequest) {
  try {
    const { sessionId, type, message, includeLogs } = await request.json();

    if (!sessionId) {
      return NextResponse.json({ error: 'sessionId required' }, { status: 400 });
    }

    if (!type || !message) {
      return NextResponse.json({ error: 'type and message required' }, { status: 400 });
    }

    const manager = getSessionManager();
    await manager.uploadFeedback(sessionId, { type, message, includeLogs });

    return NextResponse.json({ message: 'Feedback submitted successfully' });
  } catch (error) {
    console.error('[API] Failed to submit feedback:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Failed to submit feedback' },
      { status: 500 }
    );
  }
}
