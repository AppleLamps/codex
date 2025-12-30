import { NextRequest, NextResponse } from 'next/server';
import { getSessionManager } from '@/lib/session-manager';

/**
 * POST /api/codex/review
 * Start a code review
 */
export async function POST(request: NextRequest) {
  try {
    const { sessionId, target, baseBranch, commitSha, instructions } = await request.json();

    if (!sessionId) {
      return NextResponse.json({ error: 'sessionId required' }, { status: 400 });
    }

    if (!target) {
      return NextResponse.json({ error: 'target required (uncommitted, base, commit, or custom)' }, { status: 400 });
    }

    const manager = getSessionManager();
    const result = await manager.startReview(sessionId, {
      target,
      baseBranch,
      commitSha,
      instructions,
    });

    return NextResponse.json(result);
  } catch (error) {
    console.error('[API] Failed to start review:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Failed to start review' },
      { status: 500 }
    );
  }
}
