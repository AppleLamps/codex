import { NextRequest } from 'next/server';
import { getSessionManager } from '@/lib/session-manager';

/**
 * GET /api/codex/events
 * Server-Sent Events endpoint for streaming codex events
 */
export async function GET(request: NextRequest) {
  const { searchParams } = new URL(request.url);
  const sessionId = searchParams.get('sessionId');

  if (!sessionId) {
    return new Response('sessionId required', { status: 400 });
  }

  const manager = getSessionManager();

  // Wait for session to be ready (handles race condition with session creation)
  let session;
  try {
    session = await manager.waitForReady(sessionId, 30000);
  } catch (err) {
    const message = err instanceof Error ? err.message : 'Session error';
    if (message === 'Session not found') {
      return new Response('Session not found', { status: 404 });
    }
    if (message === 'Session initialization timeout') {
      return new Response('Session initialization timeout', { status: 408 });
    }
    return new Response(message, { status: 500 });
  }

  // Create a readable stream for SSE
  const stream = new ReadableStream({
    start(controller) {
      const encoder = new TextEncoder();

      // Send initial connection event
      controller.enqueue(encoder.encode(`data: ${JSON.stringify({ type: 'connected', sessionId })}\n\n`));

      // Event handler
      const onEvent = (event: unknown) => {
        try {
          const data = JSON.stringify(event);
          controller.enqueue(encoder.encode(`data: ${data}\n\n`));
        } catch (err) {
          console.error('[SSE] Failed to encode event:', err);
        }
      };

      // Error handler
      const onError = (error: Error) => {
        try {
          const data = JSON.stringify({ type: 'error', error: error.message });
          controller.enqueue(encoder.encode(`data: ${data}\n\n`));
        } catch (err) {
          console.error('[SSE] Failed to encode error:', err);
        }
      };

      // Exit handler
      const onExit = (info: { code: number | null; signal: string | null }) => {
        try {
          const data = JSON.stringify({ type: 'exit', ...info });
          controller.enqueue(encoder.encode(`data: ${data}\n\n`));
          controller.close();
        } catch (err) {
          console.error('[SSE] Failed to encode exit:', err);
        }
      };

      // Subscribe to events
      session.emitter.on('event', onEvent);
      session.emitter.on('error', onError);
      session.emitter.on('exit', onExit);

      // Heartbeat to keep connection alive
      const heartbeat = setInterval(() => {
        try {
          controller.enqueue(encoder.encode(`: heartbeat\n\n`));
        } catch {
          clearInterval(heartbeat);
        }
      }, 15000);

      // Cleanup on close
      request.signal.addEventListener('abort', () => {
        clearInterval(heartbeat);
        session.emitter.off('event', onEvent);
        session.emitter.off('error', onError);
        session.emitter.off('exit', onExit);
      });
    },
  });

  return new Response(stream, {
    headers: {
      'Content-Type': 'text/event-stream',
      'Cache-Control': 'no-cache',
      'Connection': 'keep-alive',
    },
  });
}
