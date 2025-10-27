import type {
  CardSummary,
  ReviewGrade,
  SessionStats,
  StartSessionResponse,
} from '../types/gateway';

/* c8 ignore start */
let baseUrlFromEnv: string | undefined;

if (typeof import.meta !== 'undefined') {
  const env: unknown = import.meta.env;
  if (typeof env === 'object' && env !== null && 'VITE_SESSION_URL' in env) {
    const envSessionUrl = (env as { VITE_SESSION_URL?: unknown }).VITE_SESSION_URL;
    if (typeof envSessionUrl === 'string') {
      baseUrlFromEnv = envSessionUrl;
    }
  }
}
const BASE_URL: string = baseUrlFromEnv ?? 'http://localhost:3000';
/* c8 ignore stop */

type JsonShape<T> = T;

type RequestConfig = Omit<RequestInit, 'body'> & {
  body?: Record<string, unknown>;
};
type RequestConfigWithBody = RequestConfig & { body: Record<string, unknown> };

const hasBody = (config: RequestConfig): config is RequestConfigWithBody =>
  config.body !== undefined;

const toRequestInit = (init: RequestConfig): RequestInit => {
  if (hasBody(init)) {
    return normalizeConfig(init);
  }

  const { body: _unusedBody, ...rest } = init;
  void _unusedBody;
  return rest;
};

async function request<T>(path: string, init: RequestConfig = {}): Promise<JsonShape<T>> {
  const config = toRequestInit(init);
  const response = await fetch(`${BASE_URL}${path}`, config);
  if (!response.ok) {
    throw new Error(`${path} failed: ${String(response.status)}`);
  }
  return (await response.json()) as T;
}

function normalizeConfig(init: RequestConfigWithBody): RequestInit {
  const headers = new Headers(init.headers);
  headers.set('content-type', 'application/json');
  const { body, ...rest } = init;
  return {
    ...rest,
    body: JSON.stringify(body),
    headers,
  } satisfies RequestInit;
}

const buildStatsPath = (sessionId: string): string => {
  const params = new URLSearchParams({ session_id: sessionId });
  return `/api/session/stats?${params.toString()}`;
};

export const sessionGateway = {
  startSession(userId: string): Promise<StartSessionResponse> {
    return request<StartSessionResponse>('/api/session/start', {
      method: 'POST',
      body: { user_id: userId },
    });
  },
  grade(
    sessionId: string,
    cardId: string,
    gradeValue: ReviewGrade,
    latencyMs: number,
  ): Promise<{ next_card?: CardSummary; stats?: SessionStats }> {
    return request<{ next_card?: CardSummary; stats?: SessionStats }>('/api/session/grade', {
      method: 'POST',
      body: {
        session_id: sessionId,
        card_id: cardId,
        grade: gradeValue,
        latency_ms: latencyMs,
      },
    });
  },
  stats(sessionId: string): Promise<SessionStats> {
    return request<SessionStats>(buildStatsPath(sessionId), { method: 'GET' });
  },
} as const;
