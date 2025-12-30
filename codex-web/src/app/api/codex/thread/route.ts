import { NextRequest, NextResponse } from 'next/server';
import { getSessionManager } from '@/lib/session-manager';

/**
 * POST /api/codex/thread
 * Start a new thread
 */
export async function POST(request: NextRequest) {
  try {
    const { sessionId, model, cwd } = await request.json();

    if (!sessionId) {
      return NextResponse.json({ error: 'sessionId required' }, { status: 400 });
    }

    const manager = getSessionManager();
    const thread = await manager.startThread(sessionId, { model, cwd });

    return NextResponse.json({ thread });
  } catch (error) {
    console.error('[API] Failed to start thread:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Failed to start thread' },
      { status: 500 }
    );
  }
}

/**
 * GET /api/codex/thread
 * List all threads
 */
export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const sessionId = searchParams.get('sessionId');

    if (!sessionId) {
      return NextResponse.json({ error: 'sessionId required' }, { status: 400 });
    }

    const manager = getSessionManager();
    const result = await manager.listThreads(sessionId);

    return NextResponse.json(result);
  } catch (error) {
    console.error('[API] Failed to list threads:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Failed to list threads' },
      { status: 500 }
    );
  }
}

/**
 * PUT /api/codex/thread
 * Resume an existing thread
 */
export async function PUT(request: NextRequest) {
  try {
    const { sessionId, threadId } = await request.json();

    if (!sessionId || !threadId) {
      return NextResponse.json({ error: 'sessionId and threadId required' }, { status: 400 });
    }

    const manager = getSessionManager();
    const thread = await manager.resumeThread(sessionId, threadId);

    return NextResponse.json({ thread });
  } catch (error) {
    console.error('[API] Failed to resume thread:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Failed to resume thread' },
      { status: 500 }
    );
  }
}

/**
 * DELETE /api/codex/thread
 * Archive a thread
 */
export async function DELETE(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const sessionId = searchParams.get('sessionId');
    const threadId = searchParams.get('threadId');

    if (!sessionId || !threadId) {
      return NextResponse.json({ error: 'sessionId and threadId required' }, { status: 400 });
    }

    const manager = getSessionManager();
    await manager.archiveThread(sessionId, threadId);

    return NextResponse.json({ message: 'Thread archived successfully' });
  } catch (error) {
    console.error('[API] Failed to archive thread:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Failed to archive thread' },
      { status: 500 }
    );
  }
}
