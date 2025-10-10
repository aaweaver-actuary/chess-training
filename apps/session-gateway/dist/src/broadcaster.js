const toJson = (payload) => JSON.stringify(payload);
const sendSafe = (socket, payload) => {
  if (socket.readyState === socket.OPEN) {
    socket.send(toJson(payload));
  }
};
export const createBroadcaster = () => {
  const channels = new Map();
  const ensureChannel = (sessionId) => {
    const existing = channels.get(sessionId);
    if (existing) {
      return existing;
    }
    const created = new Set();
    channels.set(sessionId, created);
    return created;
  };
  return {
    register(sessionId, socket) {
      ensureChannel(sessionId).add(socket);
    },
    unregister(sessionId, socket) {
      const channel = channels.get(sessionId);
      if (!channel) {
        return;
      }
      channel.delete(socket);
      if (channel.size === 0) {
        channels.delete(sessionId);
      }
    },
    broadcast(sessionId, payload) {
      const channel = channels.get(sessionId);
      if (!channel) {
        return;
      }
      channel.forEach((socket) => sendSafe(socket, payload));
    },
  };
};
