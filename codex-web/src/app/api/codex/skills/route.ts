import { NextRequest, NextResponse } from 'next/server';
import { getSessionManager } from '@/lib/session-manager';

/**
 * GET /api/codex/skills
 * List available skills
 */
export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const sessionId = searchParams.get('sessionId');

    if (!sessionId) {
      return NextResponse.json({ error: 'sessionId required' }, { status: 400 });
    }

    const manager = getSessionManager();
    const skills = await manager.listSkills(sessionId);

    return NextResponse.json(skills);
  } catch (error) {
    console.error('[API] Failed to list skills:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Failed to list skills' },
      { status: 500 }
    );
  }
}
