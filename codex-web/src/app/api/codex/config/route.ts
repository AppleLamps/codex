import { NextRequest, NextResponse } from 'next/server';
import { getSessionManager } from '@/lib/session-manager';

/**
 * GET /api/codex/config
 * Read configuration
 */
export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const sessionId = searchParams.get('sessionId');
    const key = searchParams.get('key') || undefined;

    if (!sessionId) {
      return NextResponse.json({ error: 'sessionId required' }, { status: 400 });
    }

    const manager = getSessionManager();
    const config = await manager.readConfig(sessionId, key);

    return NextResponse.json(config);
  } catch (error) {
    console.error('[API] Failed to read config:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Failed to read config' },
      { status: 500 }
    );
  }
}

/**
 * PUT /api/codex/config
 * Write configuration
 */
export async function PUT(request: NextRequest) {
  try {
    const { sessionId, key, value } = await request.json();

    if (!sessionId) {
      return NextResponse.json({ error: 'sessionId required' }, { status: 400 });
    }

    if (!key) {
      return NextResponse.json({ error: 'key required' }, { status: 400 });
    }

    const manager = getSessionManager();
    await manager.writeConfig(sessionId, key, value);

    return NextResponse.json({ message: 'Config updated', key, value });
  } catch (error) {
    console.error('[API] Failed to write config:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Failed to write config' },
      { status: 500 }
    );
  }
}
