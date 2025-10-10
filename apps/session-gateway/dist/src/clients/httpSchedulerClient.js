const ensureOk = async (response) => {
    if (!response.ok) {
        const message = await response.text();
        throw new Error(`scheduler-error: ${response.status} ${message}`);
    }
};
const parseJson = async (response) => {
    const text = await response.text();
    return text.length ? JSON.parse(text) : {};
};
/**
 * Create a scheduler client that talks to the HTTP scheduler service.
 */
export const createHttpSchedulerClient = (options) => {
    const fetcher = options.fetchImpl ?? fetch;
    const queueUrl = `${options.baseUrl}/queue`;
    const gradeUrl = `${options.baseUrl}/grade`;
    return {
        async fetchQueue(userId) {
            const response = await fetcher(queueUrl, {
                method: 'POST',
                headers: { 'content-type': 'application/json' },
                body: JSON.stringify({ user_id: userId }),
            });
            await ensureOk(response);
            const payload = await parseJson(response);
            return payload.queue ?? [];
        },
        async gradeCard({ sessionId, cardId, grade, latencyMs }) {
            const response = await fetcher(gradeUrl, {
                method: 'POST',
                headers: { 'content-type': 'application/json' },
                body: JSON.stringify({
                    session_id: sessionId,
                    card_id: cardId,
                    grade,
                    latency_ms: latencyMs,
                }),
            });
            await ensureOk(response);
            const payload = await parseJson(response);
            return payload.next_card ?? null;
        },
    };
};
