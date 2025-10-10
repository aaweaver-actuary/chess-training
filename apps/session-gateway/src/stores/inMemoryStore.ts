import { SessionStore } from '../types.js';

/**
 * Create a simple in-memory session store suitable for tests and local development.
 */
export const createInMemorySessionStore = <T>(): SessionStore<T> => {
  const sessions = new Map<string, T>();
  return {
    async create(sessionId, value) {
      sessions.set(sessionId, value);
    },
    async get(sessionId) {
      return sessions.get(sessionId);
    },
    async update(sessionId, updater) {
      const current = sessions.get(sessionId);
      if (!current) {
        throw new Error('session-missing');
      }
      const updated = updater(current);
      sessions.set(sessionId, updated);
      return updated;
    },
    async delete(sessionId) {
      sessions.delete(sessionId);
    },
  };
};
