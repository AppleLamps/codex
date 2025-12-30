import { NextRequest, NextResponse } from 'next/server';
import { getSessionManager } from '@/lib/session-manager';

/**
 * GET /api/codex/auth
 * Get authentication status
 */
export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const sessionId = searchParams.get('sessionId');

    if (!sessionId) {
      return NextResponse.json({ error: 'sessionId required' }, { status: 400 });
    }

    const manager = getSessionManager();
    const status = await manager.getAuthStatus(sessionId);

    return NextResponse.json(status);
  } catch (error) {
    console.error('[API] Failed to get auth status:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Failed to get auth status' },
      { status: 500 }
    );
  }
}

/**
 * POST /api/codex/auth
 * Login (API key or device auth)
 */
export async function POST(request: NextRequest) {
  try {
    const { sessionId, method, apiKey } = await request.json();

    if (!sessionId) {
      return NextResponse.json({ error: 'sessionId required' }, { status: 400 });
    }

    if (!method) {
      return NextResponse.json({ error: 'method required (apiKey or device)' }, { status: 400 });
    }

    const manager = getSessionManager();

    if (method === 'apiKey') {
      if (!apiKey) {
        return NextResponse.json({ error: 'apiKey required for apiKey method' }, { status: 400 });
      }
      const result = await manager.loginApiKey(sessionId, apiKey);
      return NextResponse.json(result);
    }

    if (method === 'device') {
      const result = await manager.loginDevice(sessionId);
      return NextResponse.json(result);
    }

    return NextResponse.json({ error: 'Invalid method' }, { status: 400 });
  } catch (error) {
    console.error('[API] Failed to login:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Failed to login' },
      { status: 500 }
    );
  }
}

/**
 * DELETE /api/codex/auth
 * Logout or cancel device auth
 */
export async function DELETE(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const sessionId = searchParams.get('sessionId');
    const action = searchParams.get('action') || 'logout';

    if (!sessionId) {
      return NextResponse.json({ error: 'sessionId required' }, { status: 400 });
    }

    const manager = getSessionManager();

    if (action === 'cancelDevice') {
      await manager.cancelLoginDevice(sessionId);
      return NextResponse.json({ message: 'Device auth cancelled' });
    }

    await manager.logout(sessionId);
    return NextResponse.json({ message: 'Logged out successfully' });
  } catch (error) {
    console.error('[API] Failed to logout:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Failed to logout' },
      { status: 500 }
    );
  }
}
