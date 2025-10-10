export const createInMemorySessionStore = () => {
  const sessions = new Map();
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
