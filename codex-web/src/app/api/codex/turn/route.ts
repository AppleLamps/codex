import { NextRequest, NextResponse } from 'next/server';
import { getSessionManager } from '@/lib/session-manager';
import type { UserInput } from '@/types/codex';

/**
 * POST /api/codex/turn
 * Start a new turn (send user message)
 */
export async function POST(request: NextRequest) {
  try {
    const { sessionId, threadId, input, model, cwd } = await request.json();

    if (!sessionId || !threadId) {
      return NextResponse.json({ error: 'sessionId and threadId required' }, { status: 400 });
    }

    if (!input) {
      return NextResponse.json({ error: 'input required' }, { status: 400 });
    }

    // Normalize input to array format
    const normalizedInput: UserInput[] = typeof input === 'string'
      ? [{ type: 'text', text: input }]
      : input;

    const manager = getSessionManager();
    const turn = await manager.startTurn(sessionId, {
      threadId,
      input: normalizedInput,
      model,
      cwd,
    });

    return NextResponse.json({ turn });
  } catch (error) {
    console.error('[API] Failed to start turn:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Failed to start turn' },
      { status: 500 }
    );
  }
}

/**
 * DELETE /api/codex/turn
 * Interrupt the current turn
 */
export async function DELETE(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const sessionId = searchParams.get('sessionId');

    if (!sessionId) {
      return NextResponse.json({ error: 'sessionId required' }, { status: 400 });
    }

    const manager = getSessionManager();
    await manager.interruptTurn(sessionId);

    return NextResponse.json({ message: 'Turn interrupted' });
  } catch (error) {
    console.error('[API] Failed to interrupt turn:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Failed to interrupt turn' },
      { status: 500 }
    );
  }
}
