import { NextRequest, NextResponse } from 'next/server';
import { getSessionManager } from '@/lib/session-manager';

/**
 * GET /api/codex/models
 * List available models
 */
export async function GET(request: NextRequest) {
  try {
    const { searchParams } = new URL(request.url);
    const sessionId = searchParams.get('sessionId');

    if (!sessionId) {
      return NextResponse.json({ error: 'sessionId required' }, { status: 400 });
    }

    const manager = getSessionManager();
    const models = await manager.listModels(sessionId);

    return NextResponse.json(models);
  } catch (error) {
    console.error('[API] Failed to list models:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Failed to list models' },
      { status: 500 }
    );
  }
}

/**
 * PUT /api/codex/models
 * Set default model
 */
export async function PUT(request: NextRequest) {
  try {
    const { sessionId, model } = await request.json();

    if (!sessionId) {
      return NextResponse.json({ error: 'sessionId required' }, { status: 400 });
    }

    if (!model) {
      return NextResponse.json({ error: 'model required' }, { status: 400 });
    }

    const manager = getSessionManager();
    await manager.setDefaultModel(sessionId, model);

    return NextResponse.json({ message: 'Default model set', model });
  } catch (error) {
    console.error('[API] Failed to set default model:', error);
    return NextResponse.json(
      { error: error instanceof Error ? error.message : 'Failed to set default model' },
      { status: 500 }
    );
  }
}
