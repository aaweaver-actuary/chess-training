import { afterEach, describe, expect, it, vi } from 'vitest';
import request from 'supertest';
import { WebSocket } from 'ws';
import http from 'http';
import { createGatewayServer } from '../src/server.js';
const italianCards = [
    {
        card_id: 'c123',
        kind: 'Opening',
        position_fen: 'r1bqkbnr/pppp1ppp/2n5/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 3 3',
        prompt: 'White to move',
        meta: { repertoire: 'Italian Game', depth: 2 },
    },
    {
        card_id: 'c456',
        kind: 'Tactic',
        position_fen: '3r2k1/pp3ppp/2p2q2/2Pp4/3P4/2N1P1P1/PP3PBP/4Q1K1 w - - 0 1',
        prompt: 'Find the best move',
    },
];
class InMemoryScheduler {
    index = 0;
    grades = [];
    async fetchQueue() {
        return italianCards;
    }
    async gradeCard({ grade }) {
        this.grades.push(grade);
        this.index += 1;
        return italianCards[this.index] ?? null;
    }
    getSubmittedGrades() {
        return this.grades;
    }
}
class InMemorySessionStore {
    sessions = new Map();
    async create(sessionId, value) {
        this.sessions.set(sessionId, value);
    }
    async get(sessionId) {
        return this.sessions.get(sessionId);
    }
    async update(sessionId, updater) {
        const existing = this.sessions.get(sessionId);
        if (!existing) {
            throw new Error('session missing');
        }
        const updated = updater(existing);
        this.sessions.set(sessionId, updated);
        return updated;
    }
    async delete(sessionId) {
        this.sessions.delete(sessionId);
    }
}
const createDependencies = () => ({
    scheduler: new InMemoryScheduler(),
    store: new InMemorySessionStore(),
});
const createEmptyQueueScheduler = () => ({
    async fetchQueue() {
        return [];
    },
    async gradeCard() {
        return null;
    },
});
const selectDependencies = (overrides = {}) => {
    const defaults = createDependencies();
    return {
        scheduler: overrides.scheduler ?? defaults.scheduler,
        store: overrides.store ?? defaults.store,
    };
};
const startGateway = async (overrides = {}) => {
    const deps = selectDependencies(overrides);
    const { app, wsServer } = createGatewayServer({
        schedulerClient: deps.scheduler,
        sessionStore: deps.store,
    });
    const server = http.createServer(app);
    wsServer.attach(server);
    await new Promise((resolve) => {
        server.listen(0, resolve);
    });
    const address = server.address();
    const baseUrl = `http://127.0.0.1:${address.port}`;
    return { server, baseUrl, deps, wsServer };
};
const closeGateway = async (server) => {
    await new Promise((resolve) => server.close(() => resolve()));
};
const wait = (ms = 100) => new Promise((resolve) => setTimeout(resolve, ms));
const createWsUrl = (baseUrl, sessionId) => baseUrl.replace('http', 'ws') + `/ws?session_id=${sessionId}`;
const createSessionSocket = (baseUrl, sessionId) => new WebSocket(createWsUrl(baseUrl, sessionId));
const waitForOpen = (socket) => new Promise((resolve) => socket.once('open', resolve));
const waitForClose = (socket) => new Promise((resolve) => socket.once('close', resolve));
const startSession = async (baseUrl, userId = 'andy') => {
    return request(baseUrl)
        .post('/api/session/start')
        .send({ user_id: userId })
        .expect(200);
};
const gradeCard = async (baseUrl, sessionId, cardId, grade, latency = 2100) => {
    return request(baseUrl)
        .post('/api/session/grade')
        .send({ session_id: sessionId, card_id: cardId, grade, latency_ms: latency })
        .expect(200);
};
describe('session gateway', () => {
    let server;
    let baseUrl;
    afterEach(async () => {
        if (server) {
            await closeGateway(server);
            server = undefined;
        }
    });
    it('starts a session and returns the first card', async () => {
        ({ server, baseUrl } = await startGateway());
        const startResponse = await startSession(baseUrl);
        expect(startResponse.body.session_id).toBeDefined();
        expect(startResponse.body.queue_size).toBe(italianCards.length);
        expect(startResponse.body.first_card).toMatchObject({
            card_id: 'c123',
            prompt: 'White to move',
        });
    });
    it('returns null when the scheduler queue is empty', async () => {
        ({ server, baseUrl } = await startGateway({
            scheduler: createEmptyQueueScheduler(),
        }));
        const response = await startSession(baseUrl);
        expect(response.body.first_card).toBeNull();
    });
    it('rejects session start requests with missing user id', async () => {
        ({ server, baseUrl } = await startGateway());
        await request(baseUrl).post('/api/session/start').send({}).expect(400);
    });
    it('grades a card and responds with the next card and updated stats', async () => {
        ({ server, baseUrl } = await startGateway());
        const startResponse = await startSession(baseUrl);
        const gradeResponse = await gradeCard(baseUrl, startResponse.body.session_id, 'c123', 'Good');
        expect(gradeResponse.body.next_card.card_id).toBe('c456');
        expect(gradeResponse.body.stats.reviews_today).toBe(1);
        expect(gradeResponse.body.stats.accuracy).toBeCloseTo(1);
    });
    it('rejects grading with an invalid card id for an existing session', async () => {
        ({ server, baseUrl } = await startGateway());
        const startResponse = await startSession(baseUrl);
        const invalidCardId = 'invalid_card_id';
        const response = await request(baseUrl).post('/api/session/grade').send({
            session_id: startResponse.body.session_id,
            card_id: invalidCardId,
            grade: 'Good',
        });
        expect(response.status).toBeGreaterThanOrEqual(400);
        expect(response.body).toHaveProperty('error');
    });
    it('returns aggregated session stats', async () => {
        ({ server, baseUrl } = await startGateway());
        const startResponse = await startSession(baseUrl);
        await gradeCard(baseUrl, startResponse.body.session_id, 'c123', 'Good');
        const statsResponse = await request(baseUrl)
            .get('/api/session/stats')
            .query({ session_id: startResponse.body.session_id })
            .expect(200);
        expect(statsResponse.body).toMatchObject({
            reviews_today: 1,
            accuracy: 1,
            avg_latency_ms: 2100,
        });
    });
    it('treats an Again grade as incorrect in accuracy calculations', async () => {
        ({ server, baseUrl } = await startGateway());
        const startResponse = await startSession(baseUrl);
        const sessionId = startResponse.body.session_id;
        await gradeCard(baseUrl, sessionId, 'c123', 'Again');
        const statsResponse = await request(baseUrl)
            .get('/api/session/stats')
            .query({ session_id: sessionId })
            .expect(200);
        expect(statsResponse.body).toMatchObject({
            reviews_today: 1,
            accuracy: 0,
        });
    });
    it('broadcasts websocket updates when grades are submitted', async () => {
        ({ server, baseUrl } = await startGateway());
        const startResponse = await startSession(baseUrl);
        const sessionId = startResponse.body.session_id;
        const socket = createSessionSocket(baseUrl, sessionId);
        const messages = [];
        socket.on('message', (data) => {
            messages.push(JSON.parse(data.toString()));
        });
        await waitForOpen(socket);
        await gradeCard(baseUrl, sessionId, 'c123', 'Good');
        await new Promise((resolve) => setTimeout(resolve, 50));
        const initialUpdateMessage = messages.find((msg) => msg.type === 'UPDATE');
        expect(initialUpdateMessage).toBeTruthy();
        expect(initialUpdateMessage).toMatchObject({
            type: 'UPDATE',
            card: expect.objectContaining({ card_id: 'c456' }),
            stats: expect.objectContaining({ reviews_today: 1 }),
        });
        expect(initialUpdateMessage?.stats).toBeTruthy();
        await wait();
        const allUpdateMessages = messages.filter((msg) => msg.type === 'UPDATE');
        const statsUpdateMessage = allUpdateMessages.length > 1 ? allUpdateMessages[1] : allUpdateMessages[0];
        expect(statsUpdateMessage).toBeTruthy();
        expect(statsUpdateMessage?.stats).toBeTruthy();
        socket.close();
        await waitForClose(socket);
    });
    it('does not deliver messages to websocket clients after session ends', async () => {
        ({ server, baseUrl } = await startGateway());
        const startResponse = await startSession(baseUrl);
        const sessionId = startResponse.body.session_id;
        const wsUrl = createWsUrl(baseUrl, sessionId);
        // End the session
        await request(baseUrl)
            .post('/api/session/end')
            .send({ session_id: sessionId })
            .expect(200);
        // Reconnect WebSocket client after session end
        const socket = new WebSocket(wsUrl);
        let closed = false;
        let receivedMessage = false;
        socket.on('message', () => {
            receivedMessage = true;
        });
        await waitForOpen(socket);
        // Wait for a short period to see if any messages are delivered
        await wait();
        socket.on('close', () => {
            closed = true;
        });
        // Wait for the close event or timeout
        await wait(200);
        expect(receivedMessage).toBe(false);
        expect(closed).toBe(false);
        socket.close();
        await waitForClose(socket);
        expect(closed).toBe(true);
    });
    it('terminates the session and notifies websocket clients', async () => {
        ({ server, baseUrl } = await startGateway());
        const startResponse = await startSession(baseUrl);
        const sessionId = startResponse.body.session_id;
        const socket = createSessionSocket(baseUrl, sessionId);
        const received = [];
        socket.on('message', (data) => {
            received.push(JSON.parse(data.toString()));
        });
        await waitForOpen(socket);
        await request(baseUrl)
            .post('/api/session/end')
            .send({ session_id: sessionId })
            .expect(200);
        await wait();
        expect(received.some((msg) => msg.type === 'SESSION_END')).toBe(true);
        socket.close();
        await waitForClose(socket);
    });
    it('destroys websocket upgrades without a session id', async () => {
        ({ server, baseUrl } = await startGateway());
        const firstDestroy = vi.fn();
        const secondDestroy = vi.fn();
        const invalidRequest = {
            url: ':::',
            headers: { host: '::invalid::' },
        };
        const missingSessionRequest = {
            url: '/ws',
            headers: {},
        };
        server.emit('upgrade', invalidRequest, { destroy: firstDestroy }, Buffer.alloc(0));
        server.emit('upgrade', missingSessionRequest, { destroy: secondDestroy }, Buffer.alloc(0));
        expect(firstDestroy).toHaveBeenCalled();
        expect(secondDestroy).toHaveBeenCalled();
    });
    it('validates inputs and rejects unknown sessions', async () => {
        ({ server, baseUrl } = await startGateway());
        await request(baseUrl)
            .post('/api/session/grade')
            .send({ session_id: 'missing', card_id: 'c1', grade: 'Bad' })
            .expect(400);
        await request(baseUrl)
            .post('/api/session/grade')
            .send({ session_id: 'missing', card_id: 'c1', grade: 'Good', latency_ms: 1200 })
            .expect(400);
        await request(baseUrl).get('/api/session/stats').expect(400);
        await request(baseUrl)
            .get('/api/session/stats')
            .query({ session_id: 'missing' })
            .expect(404);
        await request(baseUrl).post('/api/session/end').send({}).expect(400);
    });
    it('reports healthy status via the health endpoint', async () => {
        ({ server, baseUrl } = await startGateway());
        const response = await request(baseUrl).get('/api/health').expect(200);
        expect(response.body).toEqual({ status: 'ok' });
    });
    it('returns 400 for invalid session errors but rethrows other errors as 500', async () => {
        // Test invalid session returns 400
        ({ server, baseUrl } = await startGateway());
        const response = await request(baseUrl)
            .post('/api/session/grade')
            .send({ session_id: 'nonexistent', card_id: 'c1', grade: 'Good', latency_ms: 1000 })
            .expect(400);
        expect(response.body.error.code).toBe('INVALID_SESSION');
        // Test other errors are not caught and return 500
        // Create a scheduler that throws a non-session error
        const failingScheduler = {
            async fetchQueue() {
                return italianCards;
            },
            async gradeCard() {
                throw new Error('Database connection failed');
            },
        };
        const failing = await startGateway({
            scheduler: failingScheduler,
        });
        const failingBaseUrl = failing.baseUrl;
        const failingStartResponse = await request(failingBaseUrl)
            .post('/api/session/start')
            .send({ user_id: 'test' })
            .expect(200);
        // This should result in a 500 error, not a 400
        await request(failingBaseUrl)
            .post('/api/session/grade')
            .send({
            session_id: failingStartResponse.body.session_id,
            card_id: 'c123',
            grade: 'Good',
            latency_ms: 1000,
        })
            .expect(500);
        await closeGateway(failing.server);
    });
});
