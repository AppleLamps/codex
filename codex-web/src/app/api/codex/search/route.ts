import { NextRequest, NextResponse } from 'next/server';
import { getSessionManager } from '@/lib/session-manager';

/**
 * GET /api/codex/search
 * Fuzzy file search
 */
export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const sessionId = searchParams.get('sessionId');
    const query = searchParams.get('query');
    const limit = searchParams.get('limit');

    if (!sessionId) {
      return NextResponse.json({ error: 'sessionId required' }, { status: 400 });
    }

    if (!query) {
      return NextResponse.json({ error: 'query required' }, { status: 400 });
    }

    const manager = getSessionManager();
    const results = await manager.fuzzyFileSearch(
      sessionId,
      query,
      limit ? parseInt(limit, 10) : undefined
    );

    return NextResponse.json(results);
  } catch (error) {
    console.error('[API] Failed to search:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Failed to search' },
      { status: 500 }
    );
  }
}
