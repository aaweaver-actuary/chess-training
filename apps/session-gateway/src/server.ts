import express from 'express';
import type { RequestHandler } from 'express';
import { WebSocketServer } from 'ws';
import { z } from 'zod';
import { createBroadcaster } from './broadcaster.js';
import { createSessionService } from './sessionService.js';
import type { GatewayDependencies } from './types.js';

const startSchema = z.object({
  user_id: z.string().min(1),
});

const gradeSchema = z.object({
  session_id: z.string().min(1),
  card_id: z.string().min(1),
  grade: z.enum(['Again', 'Hard', 'Good', 'Easy']),
  latency_ms: z.number().int().positive(),
});

const statsSchema = z.object({
  session_id: z.string().min(1),
});

const endSchema = z.object({
  session_id: z.string().min(1),
});

const jsonError = (code: string, message: string) => ({
  error: { code, message },
});

const asyncHandler =
  (handler: RequestHandler): RequestHandler =>
  (req, res, next) => {
    Promise.resolve(handler(req, res, next)).catch(next);
  };

export const createGatewayServer = (deps: GatewayDependencies) => {
  const broadcaster = createBroadcaster();
  const service = createSessionService({ ...deps, broadcaster });
  const app = express();
  app.use(express.json());

  app.post(
    '/api/session/start',
    asyncHandler(async (req, res) => {
      const parsed = startSchema.safeParse(req.body);
      if (!parsed.success) {
        res.status(400).json(jsonError('BAD_REQUEST', 'Invalid user_id'));
        return;
      }
      const { sessionId, queueSize, firstCard } = await service.start(
        parsed.data.user_id,
      );
      res.json({ session_id: sessionId, queue_size: queueSize, first_card: firstCard });
    }),
  );

  app.post(
    '/api/session/grade',
    asyncHandler(async (req, res) => {
      const parsed = gradeSchema.safeParse(req.body);
      if (!parsed.success) {
        res.status(400).json(jsonError('BAD_REQUEST', 'Invalid grade payload supplied'));
        return;
      }
      try {
        const { nextCard, stats } = await service.grade({
          sessionId: parsed.data.session_id,
          cardId: parsed.data.card_id,
          grade: parsed.data.grade,
          latencyMs: parsed.data.latency_ms,
        });
        res.json({ next_card: nextCard, stats });
      } catch {
        res
          .status(400)
          .json(jsonError('INVALID_SESSION', 'Session not found or mismatched card'));
      }
    }),
  );

  app.get(
    '/api/session/stats',
    asyncHandler(async (req, res) => {
      const parsed = statsSchema.safeParse(req.query);
      if (!parsed.success) {
        res.status(400).json(jsonError('BAD_REQUEST', 'Missing session_id'));
        return;
      }
      const stats = await service.stats(parsed.data.session_id);
      if (!stats) {
        res.status(404).json(jsonError('INVALID_SESSION', 'Session not found'));
        return;
      }
      res.json(stats);
    }),
  );

  app.post(
    '/api/session/end',
    asyncHandler(async (req, res) => {
      const parsed = endSchema.safeParse(req.body);
      if (!parsed.success) {
        res.status(400).json(jsonError('BAD_REQUEST', 'Missing session_id'));
        return;
      }
      await service.end(parsed.data.session_id);
      res.json({ session_id: parsed.data.session_id, completed: true });
    }),
  );

  app.get('/api/health', (_req, res) => {
    res.json({ status: 'ok' });
  });

  const wsServer = new WebSocketServer({ noServer: true });

  const attach = (server: import('http').Server) => {
    server.on('upgrade', (request, socket, head) => {
      const { url } = request;
      const host = request.headers.host ?? 'localhost';
      const sessionId = (() => {
        try {
          /* c8 ignore next */
          const parsedUrl = new URL(url ?? '', `http://${host}`);
          return parsedUrl.searchParams.get('session_id');
        } catch {
          return null;
        }
      })();
      if (!sessionId) {
        socket.destroy();
        return;
      }
      wsServer.handleUpgrade(request, socket, head, (ws) => {
        broadcaster.register(sessionId, ws);
        ws.on('close', () => broadcaster.unregister(sessionId, ws));
      });
    });
  };

  return { app, wsServer: { attach } };
};
