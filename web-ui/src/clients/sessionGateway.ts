import type {
  CardSummary,
  ReviewGrade,
  SessionStats,
  StartSessionResponse,
} from '../types/gateway';

/* c8 ignore start */
const env = typeof import.meta !== 'undefined' ? import.meta.env : undefined;
const baseUrlFromEnv =
  env && typeof env.VITE_SESSION_URL === 'string' ? env.VITE_SESSION_URL : undefined;
const BASE_URL: string = baseUrlFromEnv ?? 'http://localhost:3000';

type JsonShape<T> = T;

type RequestConfig = Omit<RequestInit, 'body'> & { body?: Record<string, unknown> };

const toRequestInit = (init: RequestConfig): RequestInit => {
  if (init.body) {
    return normalizeConfig(init);
  }

  const rest = { ...init };
  delete rest.body;
  return rest;
};

async function request<T>(path: string, init?: RequestConfig): Promise<JsonShape<T>> {
  const config = init ? toRequestInit(init) : undefined;
  const response = await fetch(`${BASE_URL}${path}`, config);
  if (!response.ok) {
    throw new Error(`${path} failed: ${String(response.status)}`);
  }
  return (await response.json()) as T;
}

function normalizeConfig(init: RequestConfig): RequestInit {
  const headers = new Headers(init.headers);
  headers.set('content-type', 'application/json');
  return {
    ...init,
    body: JSON.stringify(init.body),
    headers,
  } satisfies RequestInit;
}

export const sessionGateway = {
  startSession(userId: string): Promise<StartSessionResponse> {
    return request<StartSessionResponse>('/api/session/start', {
      method: 'POST',
      body: { user_id: userId },
    });
  },
  grade(
    cardId: string,
    gradeValue: ReviewGrade,
    latencyMs: number,
  ): Promise<{ next_card?: CardSummary }> {
    return request<{ next_card?: CardSummary }>('/api/session/grade', {
      method: 'POST',
      body: { card_id: cardId, grade: gradeValue, latency_ms: latencyMs },
    });
  },
  stats(): Promise<SessionStats> {
    return request<SessionStats>('/api/session/stats');
  },
} as const;
