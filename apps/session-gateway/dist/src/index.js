import http from 'http';
import { pathToFileURL } from 'url';
import pino from 'pino';
import { createHttpSchedulerClient } from './clients/httpSchedulerClient.js';
import { loadConfig } from './config.js';
import { createGatewayServer } from './server.js';
import { createInMemorySessionStore } from './stores/inMemoryStore.js';
const createLogger = (level) => pino({ level });
/**
 * Bootstrap the session gateway HTTP and WebSocket servers using the current environment configuration.
 */
export const startGateway = async () => {
    const config = loadConfig();
    const logger = createLogger(config.LOG_LEVEL);
    const schedulerClient = createHttpSchedulerClient({ baseUrl: config.SCHEDULER_URL });
    const sessionStore = createInMemorySessionStore();
    const { app, wsServer } = createGatewayServer({
        schedulerClient,
        sessionStore,
        logger: {
            info: (message, context) => logger.info(context ?? {}, message),
            warn: (message, context) => logger.warn(context ?? {}, message),
            error: (message, context) => logger.error(context ?? {}, message),
        },
    });
    const server = http.createServer(app);
    wsServer.attach(server);
    await new Promise((resolve) => server.listen(config.PORT, resolve));
    const address = server.address();
    logger.info({ port: typeof address === 'object' && address ? address.port : config.PORT }, 'session-gateway-listening');
    return server;
};
const isMain = () => {
    const mainUrl = pathToFileURL(process.argv[1] ?? '').href;
    return import.meta.url === mainUrl;
};
if (isMain()) {
    /* c8 ignore start */
    startGateway().catch((error) => {
        createLogger('error').error({ err: error }, 'gateway-start-failed');
        process.exit(1);
    });
    /* c8 ignore end */
}
