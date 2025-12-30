import { NextRequest, NextResponse } from 'next/server';
import { v4 as uuidv4 } from 'uuid';
import { getSessionManager } from '@/lib/session-manager';

/**
 * POST /api/codex/session
 * Create a new session or resume an existing one
 */
export async function POST(request: NextRequest) {
  try {
    // Parse optional sessionId from body
    const body = await request.json().catch(() => ({}));
    const existingSessionId = body.sessionId;

    const manager = getSessionManager();

    // If sessionId provided and session exists, reuse it
    if (existingSessionId && manager.hasSession(existingSessionId)) {
      return NextResponse.json({
        sessionId: existingSessionId,
        reused: true,
        message: 'Session resumed successfully'
      });
    }

    // Create new session
    const sessionId = uuidv4();
    await manager.createSession(sessionId);

    return NextResponse.json({
      sessionId,
      reused: false,
      message: 'Session created successfully'
    });
  } catch (error) {
    console.error('[API] Failed to create session:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Failed to create session' },
      { status: 500 }
    );
  }
}

/**
 * DELETE /api/codex/session
 * Delete a session
 */
export async function DELETE(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const sessionId = searchParams.get('sessionId');

    if (!sessionId) {
      return NextResponse.json({ error: 'sessionId required' }, { status: 400 });
    }

    const manager = getSessionManager();
    manager.deleteSession(sessionId);

    return NextResponse.json({ message: 'Session deleted' });
  } catch (error) {
    console.error('[API] Failed to delete session:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Failed to delete session' },
      { status: 500 }
    );
  }
}
