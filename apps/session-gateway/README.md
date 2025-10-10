# Session Gateway

The session gateway is a lightweight Express + WebSocket service that proxies requests between the browser front-end and the Rust-based scheduler. It maintains ephemeral session state, coordinates review actions, and streams updates to connected clients.

## Features

- **REST API** for starting sessions, grading cards, fetching stats, and ending sessions.
- **WebSocket updates** so clients receive the next card and refreshed statistics in real time.
- **Configurable scheduler client** that forwards requests to the `scheduler-core` HTTP surface.
- **In-memory session store** for development and testing.

## Getting Started

```bash
npm install
npm run build      # or `npm run dev` for live reload
npm start
```

Environment variables are loaded directly from `process.env`:

| Variable        | Description                             | Default                 |
| --------------- | --------------------------------------- | ----------------------- |
| `PORT`          | HTTP port to listen on.                 | `3000`                  |
| `SCHEDULER_URL` | Base URL for the scheduler service.     | `http://localhost:4000` |
| `LOG_LEVEL`     | Pino log level (`info`, `debug`, etc.). | `info`                  |

## Testing

```bash
npm run test
```

Vitest covers request handlers, service logic, and the HTTP scheduler client. Add new tests alongside the code in `src/**/__tests__`.

## Project Structure

```
src/
├── broadcaster.ts          # WebSocket fan-out helper
├── clients/                # Scheduler client implementations
├── config.ts               # Environment parsing and validation
├── server.ts               # Express routes and WebSocket wiring
├── sessionService.ts       # Core session orchestration
├── stores/                 # Session persistence adapters
└── types.ts                # Shared TypeScript contracts
```

Each subdirectory includes a README when additional documentation is required.
